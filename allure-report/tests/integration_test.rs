mod some;

#[cfg(test)]
mod test {
    use crate::some::{Server, Test};
    use std::net::SocketAddr;

    use macros::{allure_step, allure_test};
    use reqwest::Method;

    use anyhow;
    use untitled::reporter::models::Attachment;
    use untitled::TestHelper;

    #[allure_test(test_name = "hello", test_description = "this-that")]
    async fn test_macro(test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let server = Server::new(3000);
        let addr = server.addr.clone();
        tokio::task::spawn(async {
            server.serve().await;
            eprintln!("Exit server");
        });

        step_1(addr, test_helper).await.unwrap();
        step_2(addr, test_helper).await;
    }

    #[allure_step(step_description = "test it works")]
    async fn step_1(addr: SocketAddr, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let client = test_helper.client();
        let builder = client.request(Method::GET, format!("http://{}/", addr));
        let _res = client.execute(builder.build().unwrap()).await?;
        Ok(())
    }

    #[allure_step(step_description = "test it works with json")]
    async fn step_2(addr: SocketAddr, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let client = test_helper.client();
        let builder = client
            .request(Method::GET, format!("http://{}/json", addr))
            .json(&Test {
                a: "XYZ".to_string(),
            });

        let _res = client.execute(builder.build().unwrap()).await.unwrap();
        let asserter = test_helper.asserter();
        asserter
            .assert_that(serde_json::json!( {
                "b": "XZY",
                "a": "XYZ"
            }))
            .is_equals_to(serde_json::json!( {
                "a": "XaZ",
                "b": "XZY"
            }))
            .await?;
        // TestHelper::equal_json(
        // serde_json::json!( {
        //     "b": "XZY",
        //     "a": "XYZ"
        // }),
        // &serde_json::json!( {
        //     "a": "XaZ",
        //     "b": "XZY"
        // }),
        // )?;
        Ok(())
    }
}
