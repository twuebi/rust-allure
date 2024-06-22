use crate::middleware::AllureConnectorMiddleware;
use crate::TestHelper;
use allure_models::{Attachment, Status, TestResult, TestResultBuilder};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub struct Reporter {
    test: TestResultBuilder,
    rx: tokio::sync::mpsc::UnboundedReceiver<Message>,
    result_tx: tokio::sync::oneshot::Sender<TestResult>,
}

#[derive(Debug)]
pub enum Message {
    StartStep(String),
    FinalizeStep(Status),
    AddAttachment(Attachment),
    Result,
}

impl Reporter {
    pub fn new(name: &str, full_name: &str, suite: &str, allure_dir: &str) -> (Self, TestHelper) {
        let test_builder = TestResultBuilder::new(name, full_name, suite);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        let reqwest_client = Client::builder().build().unwrap();

        let client = ClientBuilder::new(reqwest_client)
            .with(AllureConnectorMiddleware::new(
                PathBuf::from(allure_dir),
                tx.clone(),
            ))
            .build();
        (
            Self {
                test: test_builder,
                rx,
                result_tx,
            },
            TestHelper {
                tx,
                result_rx: Some(result_rx),
                result: None,
                allure_dir: allure_dir.into(),
                client,
            },
        )
    }

    pub async fn task(mut self) {
        while let Some(message) = self.rx.recv().await {
            eprintln!("Received message {:?}", message);

            match message {
                Message::StartStep(name) => self.start_step(&name),
                Message::FinalizeStep(status) => self.finalize_step(status),
                Message::AddAttachment(attachment) => {
                    if let Some(cs) = self.test.current_step.as_mut() {
                        cs.attachments.push(attachment)
                    }
                }
                Message::Result => {
                    let Self {
                        test,
                        rx: _,
                        result_tx,
                    } = self;
                    let result = test.build();
                    result_tx.send(result).unwrap();
                    eprintln!("Exiting");
                    break;
                }
            }
        }
    }

    pub fn start_step(&mut self, name: &str) {
        let _ = self.test.start_step(name).unwrap();
    }

    pub fn finalize_step(&mut self, status: Status) {
        self.test.finalize_step(status)
    }

    pub fn get_result(self) -> TestResult {
        self.test.build()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mime {
    ApplicationJson,
    Txt,
}

impl Display for Mime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Mime::ApplicationJson => f.write_str("application/json"),
            Mime::Txt => f.write_str("text/plain"),
        }
    }
}

impl Mime {
    pub fn as_ext(&self) -> &'static str {
        match self {
            Mime::ApplicationJson => "json",
            Mime::Txt => "txt",
        }
    }
}
