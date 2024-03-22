mod asserter;
mod helpers;
pub mod middleware;
pub mod reporter;

use crate::asserter::{Asserter, WithoutThing};
use crate::helpers::write_attachment;
use crate::reporter::Mime;
use anyhow::anyhow;
use reporter::models::{Attachment, Status, TestResult};
use reporter::Message;
use reqwest_middleware::ClientWithMiddleware;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use uuid::Uuid;

pub mod prelude {
    pub use anyhow;
    pub use macros::{allure_step, allure_test};
    pub use reqwest;
    pub use tokio;
}

pub struct TestHelper {
    tx: UnboundedSender<Message>,
    result_rx: Option<oneshot::Receiver<TestResult>>,
    result: Option<TestResult>,
    allure_dir: String,
    client: ClientWithMiddleware,
}

impl TestHelper {
    pub fn asserter<Z, T>(&mut self) -> Asserter<Z, T, WithoutThing>
    where
        Z: PartialEq<T> + Debug,
        T: PartialEq<Z> + Debug,
    {
        return Asserter {
            helper: self,
            thing: None,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        };
    }
    pub async fn fetch_result(&mut self) -> anyhow::Result<&TestResult> {
        assert!(
            self.result_rx.is_none() && self.result.is_none(),
            "Result was fetched but never stored, something's very wrong."
        );

        self.tx.send(Message::Result)?;
        if let Some(rx) = self.result_rx.take() {
            self.result = Some(rx.await?);
        }

        Ok(self.result.as_ref().unwrap())
    }

    pub fn client(&self) -> ClientWithMiddleware {
        self.client.clone()
    }

    pub fn equals<T: PartialEq>(a: T, b: &T) -> Result<(), ()> {
        if a.eq(b) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn equal_json(
        expected: serde_json::Value,
        actual: &serde_json::Value,
    ) -> anyhow::Result<()> {
        if expected.eq(actual) {
            Ok(())
        } else {
            let expected = serde_json::to_string_pretty(&expected).unwrap();
            let actual = serde_json::to_string_pretty(&actual).unwrap();
            let diff = similar::TextDiff::from_lines(&expected, &actual);
            Err(anyhow!(diff.unified_diff().to_string()))
        }
    }

    // TODO: add description?
    pub async fn start_step(&mut self, name: &str) -> anyhow::Result<()> {
        self.tx.send(Message::StartStep(name.into()))?;
        Ok(())
    }

    // TODO: add fields?
    pub async fn finalize_step(&mut self, status: Status) -> anyhow::Result<()> {
        self.tx.send(Message::FinalizeStep(status))?;
        Ok(())
    }

    pub async fn attachment(
        &mut self,
        name: &str,
        mime: Mime,
        content: &[u8],
    ) -> anyhow::Result<()> {
        let of = write_attachment(mime, content, self.allure_dir.as_str().into()).await?;
        self.tx.send(Message::AddAttachment(Attachment {
            name: name.into(),
            source: of.into(),
            r#type: mime.to_string().to_string(),
        }))?;
        Ok(())
    }

    pub async fn write_result(&self) -> anyhow::Result<()> {
        if let Some(r) = self.result.as_ref() {
            let mut target_dir = PathBuf::from(&self.allure_dir);
            target_dir.push(format!("{}-result.json", Uuid::new_v4()));
            tokio::fs::write(target_dir, serde_json::to_string(r).unwrap()).await?;
        } else {
            anyhow::bail!("Result is not fetched, fetch result before trying to write it.");
        }
        Ok(())
    }
}
