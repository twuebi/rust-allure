# allure-rust

HTTP integration testing in Rust producing [Allure](https://docs.qameta.io/allure/) compatible outputs.

```rust
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

#[allure_step(step_description = "Test the server responds 'Hello World!'.")]
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
```

<p>
<img alt="img.png" height="550" src="img.png" width="855"/>
</p>

More examples can be found under [examples](./examples), to explore them simply:

```bash
$ cd examples
$ cargo test # or cargo nextest r
...
$ allure serve allure-results
```

and wait for your browser to open.

