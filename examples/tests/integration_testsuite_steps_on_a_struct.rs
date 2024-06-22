pub mod helpers;

use std::net::SocketAddr;

use allure_report::prelude::{anyhow, reqwest_middleware, tokio};
use allure_report::{allure_step, allure_test};

use crate::helpers::common_steps::a_shared_step;
use crate::helpers::server::{Server, Test};
use allure_report::prelude::reqwest::Method;
use allure_report::TestHelper;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

struct Tester {
    client: reqwest_middleware::ClientWithMiddleware,
    addr: SocketAddr,
}

#[allure_test(
    test_name = "Two endpoint test",
    test_description = "This is a test with a failing step"
)]
async fn test_feature_a(test_helper: &mut TestHelper) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let server = Server::new(0).await;
    let addr = server.addr;
    server.spawn_serve();

    let tester = Tester {
        client: test_helper.client().clone(),
        addr,
    };

    a_shared_step(addr, test_helper).await?;
    tester.step_2(test_helper).await?;
}

impl Tester {
    #[allure_step(step_description = "Test that the server responds with the expected JSON.")]
    async fn step_2(&self, test_helper: &mut TestHelper) -> anyhow::Result<()> {
        let client = self.client.clone();
        let builder = client
            .request(Method::POST, format!("http://{}/json", self.addr))
            .json(&Test {
                a: "XYZ".to_string(),
            });

        let res: serde_json::Value = client.execute(builder.build()?).await?.json().await?;
        let asserter = test_helper.asserter();
        asserter
            .assert_that(res)
            .is_equals_to(
                serde_json::json!( {
                    "a": "XaZ",
                    "b": "XZY"
                }),
                Some("assertion that \"b\" is \"XaZ\" and \"a\" is \"XZY\""),
            )
            .await?;
        Ok(())
    }
}

#[allure_test(test_name = "hello2", test_description = "this-that")]
async fn test_feature_b(test_helper: &mut TestHelper) -> anyhow::Result<()> {
    let server = Server::new(0).await;
    let addr = server.addr;

    server.spawn_serve();

    a_shared_step(addr, test_helper).await?;
}
