use std::path::PathBuf;
use task_local_extensions::Extensions;
use untitled::{Status, TestResult, TestResultBuilder};

mod some;

pub struct Reporter {
    extensions: Extensions,
}

impl Reporter {
    pub fn new(name: &str, full_name: &str, suite: &str) -> Self {
        let test_builder = TestResultBuilder::new(name, full_name, suite);
        let mut ext = Extensions::new();
        ext.insert(test_builder);
        Self { extensions: ext }
    }

    pub fn start_step(&mut self, name: &str) {
        let step_builder: Option<&mut TestResultBuilder> = self.extensions.get_mut();
        let _ = step_builder.unwrap().start_step(name).unwrap();
    }

    pub fn finalize_step(&mut self, status: Status) {
        let step_builder: &mut TestResultBuilder = self.extensions.get_mut().unwrap();
        let _ = step_builder.finalize_step(status);
    }

    pub fn ext(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn get_result(mut self) -> TestResult {
        let builder: TestResultBuilder = self.extensions.remove().unwrap();
        builder.build()
    }

    pub async fn write_result(result: &TestResult, target_dir: PathBuf) {
        let mut of = PathBuf::try_from(target_dir).unwrap();
        of.push(format!("{}-result.json", uuid::Uuid::new_v4()));
        tokio::fs::write(of, serde_json::to_string(result).unwrap())
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use std::net::SocketAddr;
    use crate::some::{Server, Test};
    use crate::Reporter;
    use reqwest::{Client, Method};
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Extension};
    use std::path::PathBuf;
    use task_local_extensions::Extensions;
    use untitled::middleware::LoggingMiddleware;
    use untitled::{Status, TestResultBuilder};
    use std::sync::Arc;

    #[tokio::test]
    async fn test() {
        let server = Server::new(3000);
        let addr = server.addr.clone();
        tokio::task::spawn(server.serve());
        let reqwest_client = Client::builder().build().unwrap();
        let mut reporter = Reporter::new("Test it".into(), "Test that".into(), "1");

        let client = ClientBuilder::new(reqwest_client)
            .with(LoggingMiddleware::new(
                PathBuf::try_from("allure-results").unwrap(),
            ))
            .build();
        client
            .get(format!("http://{}/", addr))
            .send()
            .await
            .unwrap();

        step_1(addr, &client, &mut reporter).await;

        let builder = client
            .request(Method::GET, format!("http://{}/json", addr))
            .json(&Test {
                a: "XYZ".to_string(),
            });
        reporter.start_step("Test it works with json".into());

        let _res = client
            .execute_with_extensions(builder.build().unwrap(), reporter.ext())
            .await
            .unwrap();
        reporter.finalize_step(Status::Failed);

        let result = reporter.get_result();
        Reporter::write_result(&result, "allure-results".try_into().unwrap()).await;
    }

    async fn step_1(addr: SocketAddr, client: &ClientWithMiddleware, mut reporter: &mut Reporter) {
        reporter.start_step("Test it works".into());
        let builder = client.request(Method::GET, format!("http://{}/", addr));
        let _res = client
            .execute_with_extensions(builder.build().unwrap(), reporter.ext())
            .await
            .unwrap();
        reporter.finalize_step(Status::Passed);
    }

    #[tokio::test]
    async fn test2() {
        let server = Server::new(3001);
        let addr = server.addr.clone();
        tokio::task::spawn(server.serve());
        let reqwest_client = Client::builder().build().unwrap();
        let client = ClientBuilder::new(reqwest_client)
            .with(LoggingMiddleware::new(
                PathBuf::try_from("allure-results").unwrap(),
            ))
            .build();
        dbg!(client
            .get(format!("http://{}/", addr))
            .send()
            .await
            .unwrap());
        let test_builder = TestResultBuilder::new("Test it2", "Test that2", "2");
        let mut ext = Extensions::new();
        ext.insert(test_builder);
        {
            let builder = client.request(Method::GET, format!("http://{}/", addr));
            {
                let step_builder: Option<&mut TestResultBuilder> = ext.get_mut();
                let _ = step_builder
                    .unwrap()
                    .start_step("Test it works".into())
                    .unwrap();
            }

            let _res = client
                .execute_with_extensions(builder.build().unwrap(), &mut ext)
                .await
                .unwrap();
            {
                let step_builder: Option<&mut TestResultBuilder> = ext.get_mut();
                let _ = step_builder.unwrap().finalize_step(Status::Passed);
            }
        }

        {
            let builder = client
                .request(Method::GET, format!("http://{}/json", addr))
                .json(&Test {
                    a: "XYZ".to_string(),
                });
            {
                let step_builder: Option<&mut TestResultBuilder> = ext.get_mut();
                let _ = step_builder
                    .unwrap()
                    .start_step("Test it works with json".into())
                    .unwrap();
            }

            let _res = client
                .execute_with_extensions(builder.build().unwrap(), &mut ext)
                .await
                .unwrap();
            {
                let step_builder: Option<&mut TestResultBuilder> = ext.get_mut();
                let _ = step_builder.unwrap().finalize_step(Status::Passed);
            }
        }
        let builde: TestResultBuilder = dbg!(ext.remove()).unwrap();
        let mut of = PathBuf::try_from("allure-results").unwrap();
        of.push(format!("{}-result.json", uuid::Uuid::new_v4()));
        tokio::fs::write(of, serde_json::to_string(&builde.build()).unwrap())
            .await
            .unwrap();
    }
}
