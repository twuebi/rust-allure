pub mod middleware;

use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use uuid::Uuid;

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
    pub fn new(
        name: &str,
        full_name: &str,
        suite: &str,
    ) -> (
        Self,
        UnboundedSender<Message>,
        oneshot::Receiver<TestResult>,
    ) {
        let test_builder = TestResultBuilder::new(name, full_name, suite);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();
        (
            Self {
                test: test_builder,
                rx,
                result_tx,
            },
            tx,
            result_rx,
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

    pub async fn write_result(result: &TestResult, mut target_dir: PathBuf) {
        target_dir.push(format!("{}-result.json", uuid::Uuid::new_v4()));
        tokio::fs::write(target_dir, serde_json::to_string(result).unwrap())
            .await
            .unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Passed,
    Failed,
    Pending,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attachment {
    name: String,
    source: PathBuf,
    // application/yaml etc
    r#type: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Step {
    pub name: String,
    pub status: Status,
    pub attachments: Vec<Attachment>,
    pub start: u128,
    pub stop: u128,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StepBuilder {
    pub name: String,
    pub attachments: Vec<Attachment>,
    pub start: u128,
}

impl StepBuilder {
    pub fn into_step(self, status: Status) -> Step {
        let StepBuilder {
            name,
            attachments,
            start,
        } = self;
        Step {
            name,
            status,
            attachments,
            start,
            stop: get_epoch_ms(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Label {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
    pub r#type: String,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestResult {
    pub uuid: Uuid,
    #[serde(serialize_with = "serialize_simple")]
    pub history_id: Uuid,
    #[serde(serialize_with = "serialize_simple")]
    pub test_case_id: Uuid,
    pub full_name: String,
    pub name: String,
    pub links: Vec<Link>,
    pub labels: Vec<Label>,
    pub status: Status,
    pub start: u128,
    pub stop: u128,
    pub steps: Vec<Step>,
    pub attachments: Vec<Attachment>,
}

impl TestResult {
    pub fn new(full_name: String, name: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            history_id: Uuid::new_v4(),
            test_case_id: Uuid::new_v4(),
            full_name,
            name,
            links: vec![],
            labels: vec![],
            status: Status::Pending,
            start: get_epoch_ms(),
            stop: 0,
            steps: vec![],
            attachments: vec![],
        }
    }
}
#[derive(Debug)]
pub struct TestResultBuilder {
    pub uuid: Uuid,
    pub full_name: String,
    pub name: String,
    pub links: Vec<Link>,
    pub labels: Vec<Label>,
    pub start: u128,
    pub current_step: Option<StepBuilder>,
    pub steps: Vec<Step>,
    pub attachments: Vec<Attachment>,
}

impl TestResultBuilder {
    pub fn new(name: &str, full_name: &str, suite: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            full_name: full_name.into(),
            name: name.into(),
            links: vec![],
            labels: vec![Label {
                name: "suite".to_string(),
                value: suite.to_string(),
            }],
            start: get_epoch_ms(),
            current_step: None,
            steps: vec![],
            attachments: vec![],
        }
    }

    pub fn start_step(&mut self, name: &str) -> anyhow::Result<&mut StepBuilder> {
        if self.current_step.is_some() {
            return Err(anyhow::anyhow!("Current step is not finalized."));
        }
        self.current_step = Some(StepBuilder {
            name: name.into(),
            attachments: vec![],
            start: get_epoch_ms(),
        });
        return Ok(self.current_step.as_mut().unwrap());
    }

    pub fn current_step(&mut self) -> Option<&mut StepBuilder> {
        self.current_step.as_mut()
    }

    pub fn finalize_step(&mut self, status: Status) {
        if let Some(step) = self.current_step.take() {
            self.steps.push(step.into_step(status))
        }
    }

    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment)
    }

    pub fn build(self) -> TestResult {
        let Self {
            uuid,
            full_name,
            name,
            links,
            labels,
            start,
            current_step: _,
            steps,
            attachments,
        } = self;
        TestResult {
            uuid,
            history_id: Uuid::new_v4(),
            test_case_id: Uuid::new_v4(),
            full_name,
            name,
            links,
            labels,
            status: if steps
                .iter()
                .map(|s| s.status)
                .any(|s| matches!(s, Status::Failed))
            {
                Status::Failed
            } else {
                Status::Passed
            },
            start,
            stop: get_epoch_ms(),
            steps,
            attachments,
        }
    }
}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn serialize_simple<S>(x: &Uuid, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.as_simple().to_string())
}
#[cfg(test)]
mod test {
    use crate::TestResult;

    #[test]
    fn test_roundtrip() {
        let val = serde_json::json!({
          "uuid": "9d95e6e7-9cf6-4ca5-91b4-9b69ce0971f8",
          "historyId": "2b35e31882061875031701ba05a3cd67",
          "testCaseId": "43f8868a367ff70177a99838d39c5b33",
          "fullName": "com.example.web.essentials.AuthenticationTest.testAuthentication",
          "name": "testAuthentication()",
          "links": [
            {
              "type": "link",
              "name": "Allure Examples",
              "url": "https://examples.com/"
            },
            {
              "type": "issue",
              "name": "BUG-123",
              "url": "https://bugs.example.com/BUG-123"
            }
          ],
          "labels": [
            {
              "name": "host",
              "value": "machine-1"
            },
            {
              "name": "thread",
              "value": "306681-MainThread"
            },
            {
              "name": "language",
              "value": "java"
            },
            {
              "name": "framework",
              "value": "junit-platform"
            },
            {
              "name": "epic",
              "value": "Web interface"
            },
            {
              "name": "feature",
              "value": "Essential features"
            },
            {
              "name": "story",
              "value": "Authentication"
            }
          ],
          "status": "passed",
          "start": 1682358426014usize,
          "stop": 1682358426014usize,
            "attachments": [],
          "steps": [
            {
              "name": "Step 1",
              "status": "passed",
              "attachments": [],
              "start": 1682358426014usize,
              "stop": 1682358426014usize
            },
            {
              "name": "Step 2",
              "status": "passed",
              "attachments": [],
              "start": 1682358426014usize,
              "stop": 1682358426014usize
            }
          ]
        });
        let parsed: TestResult = serde_json::from_value(val.clone()).unwrap();
        let val2 = serde_json::to_value(&parsed).unwrap();
        assert_eq!(val, val2);
    }
}
