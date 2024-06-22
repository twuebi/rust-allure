use allure_report::allure_step;
use allure_report::prelude::reqwest::Method;
use allure_report::prelude::*;
use allure_report::TestHelper;
use std::net::SocketAddr;

#[allure_step(step_description = "Test we can GET 'Hello, World!' from the server.")]
pub async fn a_shared_step(
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
    Ok(res)
}
