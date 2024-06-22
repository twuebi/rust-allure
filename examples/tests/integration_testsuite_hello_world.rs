pub mod helpers;

use std::net::SocketAddr;

use allure_report::prelude::*;
use allure_report::{allure_step, allure_test};

use crate::helpers::server::Server;
use allure_report::prelude::reqwest::Method;
use allure_report::TestHelper;

// Unnamed tests take the function name as the test name
#[allure_test(
    test_description = "This test makes sure that our server hello worlds in proper manner."
)]
async fn test_feature_hello_world(test_helper: &mut TestHelper) -> anyhow::Result<()> {
    let server = Server::new(0).await;
    let addr = server.addr;
    server.spawn_serve();

    make_sure_hello_world_works(addr, test_helper).await?;
}

#[allure_step(step_description = "Test the server responds 'Hello, World!'.")]
pub async fn make_sure_hello_world_works(
    addr: SocketAddr,
    test_helper: &mut TestHelper,
) -> anyhow::Result<String> {
    let client = test_helper.client();
    let builder = client.request(Method::GET, format!("http://{}/", addr));
    let res = client
        .execute(builder.build().unwrap())
        .await?
        .text()
        .await?;
    test_helper
        .asserter()
        .assert_that(&res)
        .is_equals_to("Hello World!", Some("check that we received Hello World!"))
        .await?;
    Ok(res)
}
