use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{ItemFn, ReturnType};

extern crate proc_macro;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    test_name: String,
    test_description: String,
    allure_dir: Option<String>,
}

#[proc_macro_attribute]
pub fn allure_test(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // eprintln!("attr: \"{}\"", args.to_string());
    // eprintln!("item: \"{:#?}\"", input.to_string());
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(Error::from(e).write_errors());
        }
    };
    let mut func = syn::parse_macro_input!(input as ItemFn);

    let args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(e.write_errors());
        }
    };
    func.sig.output = ReturnType::Default;
    func.sig.inputs = Punctuated::default();
    let sig = func.sig.clone().into_token_stream();

    let header = quote!(
        #[::tokio::test]
        #sig
    );
    let fname = func.sig.ident.clone();
    let allure_dir = args.allure_dir.unwrap_or("allure-results".to_string());
    let ts = args.test_name.into_token_stream();
    let desc = args.test_description.into_token_stream();

    let outer_body = quote_spanned!(func.sig.span()=> {
        let (reporter, mut test_helper) = ::untitled::Reporter::new(#ts, #desc, module_path!(), #allure_dir);
        let task_handle = ::tokio::task::spawn(reporter.task());
        inner(&mut test_helper).await;
        let result = test_helper.consume_result().await.unwrap();
        test_helper.write_result(&result).await;
    });
    // eprintln!("{outer_body}");
    let block = func.block.clone().into_token_stream();

    let body = quote_spanned!(func.block.span()=>
        async fn inner(test_helper: &mut TestHelper) -> anyhow::Result<()>
        {#block
        Ok(())}
    );

    let mut out = TokenStream::new();
    out.extend(header);
    out.extend(outer_body);
    out.extend(body);

    // eprintln!("{out}");
    out.into()
}

#[derive(Debug, FromMeta)]
struct StepArgs {
    step_description: String,
}

#[proc_macro_attribute]
pub fn allure_step(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // eprintln!("attr: \"{}\"", args.to_string());
    // eprintln!("item: \"{:#?}\"", input.to_string());
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(Error::from(e).write_errors());
        }
    };
    let func = syn::parse_macro_input!(input as ItemFn);

    let args = match StepArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(e.write_errors());
        }
    };
    let sig = func.sig.clone().into_token_stream();
    let step_name = func.sig.ident.to_string();
    let header = quote! {
        #sig
    };

    let desc = args.step_description.into_token_stream();
    let block = func.block.clone().into_token_stream();

    let body = quote_spanned!(func.block.span()=> {
        test_helper.start_step(&format!("{}: {}",#step_name, #desc)).await?;
        let client = test_helper.client();
        let clos = || async move #block;
        let res: Result<(), String> = clos().await;
        match res {
            Ok(_) => {
                test_helper.finalize_step(untitled::Status::Passed).await?;
            }
            Err(err) => {
                eprintln!("{}", err);
                test_helper.finalize_step(untitled::Status::Failed).await?;
                return Err(anyhow::anyhow!(err.to_string()))
            }
        }
        Ok(())
    });

    let mut out = TokenStream::new();
    out.extend(header);
    out.extend(body);

    // eprintln!("{out}");
    out.into()
}
