use reqwest::Url;

mod test_server;

pub struct Config {
    pub addr: Url,
    pub client: reqwest_middleware::ClientWithMiddleware,
}

#[cfg(test)]
mod test {
    use crate::test_server::{Server, Test};
    use std::fmt::format;
    use std::net::SocketAddr;

    use allure_macros::{allure_step, allure_test};
    use reqwest::Method;

    use crate::Config;
    use anyhow;
    use untitled::reporter::models::Attachment;
    use untitled::TestHelper;

    struct Tester {
        client: reqwest_middleware::ClientWithMiddleware,
        addr: SocketAddr,
    }

    #[allure_test(test_name = "hello", test_description = "this-that")]
    async fn test_macro(test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let server = Server::new(3000);
        let addr = server.addr.clone();

        let mut client = test_helper.client().clone();
        let tester = Tester {
            client: client.clone(),
            addr,
        };
        tokio::task::spawn(async {
            server.serve().await;
            eprintln!("Exit server");
        });

        step_1(addr, test_helper).await.unwrap();
        tester.step_2(test_helper).await?;
    }

    #[allure_step(step_description = "test it works")]
    async fn step_1(addr: SocketAddr, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let client = test_helper.client();
        let builder = client.request(Method::GET, format!("http://{}/", addr));
        let _res = client.execute(builder.build().unwrap()).await?;
        Ok(())
    }

    impl Tester {
        #[allure_step(step_description = "test it works with json")]
        async fn step_2(&self, test_helper: &mut TestHelper) -> anyhow::Result<()> {
            let client = self.client.clone();
            let builder = client
                .request(Method::POST, format!("http://{}/json", self.addr))
                .json(&Test {
                    a: "XYZ".to_string(),
                });

            let _res = client.execute(builder.build()?).await?;
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
            Ok(())
        }
    }
}
