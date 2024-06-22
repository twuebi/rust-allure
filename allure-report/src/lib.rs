mod asserter;
mod helpers;
pub mod middleware;
pub mod reporter;

pub mod models {
    pub use allure_models::*;
}

use crate::asserter::{Asserter, WithoutThing};
use crate::helpers::write_attachment;
use crate::reporter::Mime;
use allure_models::{Attachment, Status, TestResult};
use anyhow::anyhow;
use reporter::Message;
use reqwest_middleware::ClientWithMiddleware;
use std::fmt::Debug;

use std::path::PathBuf;

pub use allure_macros::{allure_step, allure_test};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use uuid::Uuid;

pub mod prelude {
    pub use anyhow;
    pub use reqwest;
    pub use reqwest_middleware;
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
    pub fn client(&self) -> ClientWithMiddleware {
        self.client.clone()
    }

    pub fn asserter<Z, T>(&mut self) -> Asserter<Z, T, WithoutThing>
    where
        Z: PartialEq<T> + Debug,
        T: PartialEq<Z> + Debug,
    {
        Asserter::new(self)
    }
    pub async fn ___private_fetch_result(&mut self) -> anyhow::Result<&TestResult> {
        assert!(
            !(self.result_rx.is_none() && self.result.is_none()),
            "Result was fetched but never stored, something's very wrong."
        );

        self.tx.send(Message::Result)?;
        if let Some(rx) = self.result_rx.take() {
            self.result = Some(rx.await?);
        }

        Ok(self.result.as_ref().unwrap())
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
    pub async fn ___private_start_step(&mut self, name: &str) -> anyhow::Result<()> {
        self.tx.send(Message::StartStep(name.into()))?;
        Ok(())
    }

    // TODO: add fields?
    pub async fn ___private_finalize_step(&mut self, status: Status) -> anyhow::Result<()> {
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
            source: of,
            r#type: mime.to_string().to_string(),
        }))?;
        Ok(())
    }

    pub async fn ___private_write_result(&self) -> anyhow::Result<()> {
        if let Some(r) = self.result.as_ref() {
            let mut target_dir = PathBuf::from(&self.allure_dir);
            target_dir.push(format!("{}-result.json", Uuid::now_v7()));
            tokio::fs::write(target_dir, serde_json::to_string(r).unwrap()).await?;
        } else {
            anyhow::bail!("Result is not fetched, fetch result before trying to write it.");
        }
        Ok(())
    }
}
