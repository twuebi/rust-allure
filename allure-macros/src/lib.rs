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
    let args: MacroArgs = match parse_args(args) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let mut func = syn::parse_macro_input!(input as ItemFn);

    let input_span = func.sig.paren_token.span.span();
    func.sig.output = ReturnType::Default;
    let old_inps = func.sig.inputs.clone();
    func.sig.inputs = Punctuated::default();
    let sig = func.sig.clone().into_token_stream();

    let header = quote!(
        #[::tokio::test]
        #sig
    );

    let allure_dir = args.allure_dir.unwrap_or("allure-results".to_string());
    let ts = args.test_name.into_token_stream();
    let desc = args.test_description.into_token_stream();

    let outer_body = quote_spanned!(func.block.span()=> {
        let (reporter, mut helper) = ::untitled::reporter::Reporter::new(#ts, #desc, module_path!(), #allure_dir);
        let task_handle = ::tokio::task::spawn(reporter.task());
        inner(&mut helper).await;
        let _ = helper.fetch_result().await.unwrap();
        helper.write_result().await.unwrap();
    });
    // eprintln!("{outer_body}");
    let block = func.block.clone().into_token_stream();
    let inputx = quote_spanned!(input_span=> #old_inps);
    let headerx = quote_spanned!(func.sig.span()=> async fn inner(#inputx) -> anyhow::Result<()>);

    let body = quote_spanned!(func.span()=>
        #headerx
        {#block
        Ok(())}
    );

    let mut out = TokenStream::new();
    out.extend(header);
    out.extend(outer_body);
    out.extend(body);

    eprintln!("{out}");
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
    let args: StepArgs = match parse_args(args) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let func = syn::parse_macro_input!(input as ItemFn);

    let signature = func.sig.to_token_stream();
    let step_name = func.sig.ident.to_string();

    let arguments_with_types = func.sig.inputs.to_token_stream();

    let block = TokenStream::from_iter(
        func.block
            .stmts
            .clone()
            .into_iter()
            .map(|t| t.into_token_stream()),
    );

    let (is_struct_impl, fn_inputs) = create_inner_call_arguments(&func);

    let obj = if is_struct_impl {
        quote! { self. }
    } else {
        quote! {}
    };
    let mut tokens = TokenStream::new();
    tokens.extend(
        proc_macro2::Ident::new(
            format!("{}_test_impl", func.sig.ident).as_str(),
            func.sig.span(),
        )
        .into_token_stream(),
    );

    let inner_fn_name = quote! { #tokens };

    let test_fn = quote! { async fn #inner_fn_name (#arguments_with_types) -> anyhow::Result<()> {
                                       #block
                                     }
    };
    let invocation =
        quote! { let res: anyhow::Result<()> = #obj #inner_fn_name (#(#fn_inputs),*).await; };
    let args = args.step_description.into_token_stream();
    let body = quote_spanned!(func.block.span()=> {
        test_helper.start_step(&format!("{}: {}",#step_name, #args)).await?;
        #invocation
        match res {
            Ok(_) => {
                test_helper.finalize_step(untitled::reporter::models::Status::Passed).await?;
                Ok(())
            }
            Err(err) => {
                test_helper.finalize_step(untitled::reporter::models::Status::Failed).await?;
                Err(anyhow::anyhow!(err.to_string()))
            }
        }
    });

    let mut out = TokenStream::new();
    out.extend(test_fn);
    out.extend(signature);
    out.extend(body);

    out.into()
}

fn parse_args<T: FromMeta>(args: proc_macro::TokenStream) -> Result<T, proc_macro::TokenStream> {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return Err(proc_macro::TokenStream::from(Error::from(e).write_errors()));
        }
    };

    let args = match T::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return Err(proc_macro::TokenStream::from(e.write_errors()));
        }
    };
    Ok(args)
}

fn create_inner_call_arguments(func: &ItemFn) -> (bool, Vec<TokenStream>) {
    let mut is_struct_impl = false;
    let mut fn_inputs = vec![];
    for input in func.sig.inputs.iter() {
        match input {
            syn::FnArg::Receiver(_) => {
                // This is a `self` argument
                is_struct_impl = true;
            }
            syn::FnArg::Typed(arg) => {
                if let syn::Pat::Ident(pat_ident) = &arg.pat.as_ref() {
                    // This is a regular argument, and `pat_ident.ident` is its name
                    fn_inputs.push(pat_ident.ident.to_token_stream());
                }
            }
        }
    }
    (is_struct_impl, fn_inputs)
}
