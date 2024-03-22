mod some;

#[cfg(test)]
mod test {
    use crate::some::{Server, Test};
    use std::net::SocketAddr;

    use macros::{allure_step, allure_test};
    use reqwest::Method;

    use anyhow;
    use untitled::TestHelper;

    #[allure_test(test_name = "hello", test_description = "this-that")]
    async fn test_macro(test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let server = Server::new(3000);
        let addr = server.addr.clone();
        tokio::task::spawn(async {
            server.serve().await;
            eprintln!("Exit server");
        });


        match step_1(addr, test_helper).await {
            Ok(ok) => {}
            Err(err) => {}
        }
        let _ = step_2(addr, test_helper).await;
    }

    // #[::tokio::test]
    // async fn test_macro() {
    //     use ::untitled::Reporter;
    //     let (reporter, mut test_helper) =
    //         ::untitled::Reporter::new("hello", "this-that", module_path!(), "allure-results");
    //     let task_handle = ::tokio::task::spawn(reporter.task());
    //     {
    //         let server = Server::new(3000);
    //         let addr = server.addr.clone();
    //         tokio::task::spawn(async {
    //             server.serve().await;
    //             eprintln!("Exit server");
    //         });
    //         match step_1(addr, &mut test_helper).await {
    //             Ok(ok) => {}
    //             Err(err) => {}
    //         }
    //         let _ = step_2(addr, &mut test_helper).await;
    //     }
    //     let result = test_helper.into_results().await.unwrap();
    //     Reporter::write_result(&result, "allure-results".into()).await;
    // }

    #[allure_step(step_description = "test it works with json")]
    async fn step_2(addr: SocketAddr, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let builder = client
            .request(Method::GET, format!("http://{}/json", addr))
            .json(&Test {
                a: "XYZ".to_string(),
            });
        let _res = client.execute(builder.build().unwrap()).await.unwrap();
        TestHelper::equal_json(
            serde_json::json!( {
                "b": "XZY",
                "a": "XYZ"
            }),
            &serde_json::json!( {
                "a": "XaZ",
                "b": "XZY"
            }),
        )?;
        Ok(())
    }

    #[allure_step(step_description = "test it works")]
    async fn step_1(addr: SocketAddr, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let builder = client.request(Method::GET, format!("http://{}/", addr));
        let _res = client.execute(builder.build().unwrap()).await.unwrap();
        Ok(())
    }
}
