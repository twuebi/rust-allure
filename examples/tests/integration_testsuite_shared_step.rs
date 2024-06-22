pub mod helpers;

use allure_report::allure_test;
use allure_report::prelude::*;

use crate::helpers::common_steps::a_shared_step;
use crate::helpers::server::Server;
use allure_report::TestHelper;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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

    a_shared_step(addr, test_helper).await?;
}

#[allure_test(test_name = "hello2", test_description = "this-that")]
async fn test_feature_b(test_helper: &mut TestHelper) -> anyhow::Result<()> {
    let server = Server::new(0).await;
    let addr = server.addr;

    server.spawn_serve();

    a_shared_step(addr, test_helper).await?;
}
