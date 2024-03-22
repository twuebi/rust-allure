use crate::reporter::models::Attachment;
use crate::reporter::{Message, Mime};
use http::HeaderMap;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde_json::Value;
use std::path::PathBuf;
use task_local_extensions::Extensions;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;

pub struct LoggingMiddleware {
    allure_dir: PathBuf,
    tx: UnboundedSender<Message>,
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        self.log_request(&req).await?;

        let res = next.run(req, extensions).await?;
        let (body, headers, response) = Self::prepare_response_copy(res).await?;

        self.log_response(headers, body).await?;

        Ok(response)
    }
}

impl LoggingMiddleware {
    pub fn new(allure_dir: PathBuf, tx: UnboundedSender<Message>) -> Self {
        if !allure_dir.exists() {
            std::fs::create_dir_all(&allure_dir).unwrap();
        }
        Self { allure_dir, tx }
    }

    async fn add_attachment(&self, name: &str, mime: Mime, content: Vec<u8>) -> anyhow::Result<()> {
        let of_name = self.write_attachment(mime, &content).await?;

        eprintln!("sending attachment");
        self.tx.send(Message::AddAttachment(Attachment {
            name: name.to_string(),
            source: of_name,
            r#type: mime.to_string(),
        }))?;
        Ok(())
    }

    async fn write_attachment(&self, mime: Mime, content: &Vec<u8>) -> anyhow::Result<PathBuf> {
        crate::helpers::write_attachment(mime, content, self.allure_dir.clone()).await
    }

    async fn prepare_response_copy(res: Response) -> Result<(bytes::Bytes, HeaderMap, Response)> {
        let mut copied_response = http::Response::builder().status(res.status());
        let headers = res.headers().clone();
        let body = res.bytes().await?;
        for (k, v) in headers.iter() {
            copied_response = copied_response.header(k, v);
        }
        let response = copied_response.body(body.clone()).unwrap();
        Ok((body, headers, response.into()))
    }

    async fn log_response(&self, headers: HeaderMap, body: bytes::Bytes) -> anyhow::Result<()> {
        let headers = headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, String::from_utf8_lossy(v.as_bytes())))
            .collect::<Vec<String>>();

        let body_v = if let Ok(jsn) = serde_json::from_slice(&body) {
            jsn
        } else {
            Value::String(String::from_utf8_lossy(&body).to_string())
        };

        let mut buf = Vec::new();
        for header in headers.into_iter() {
            buf.write_all(format!("{header}\n").as_bytes()).await?;
        }

        self.add_attachment("Response Headers", Mime::Txt, buf)
            .await?;

        let mut buf = Vec::new();
        serde_json::to_writer_pretty(&mut buf, &body_v).unwrap();
        self.add_attachment("Response Body", Mime::ApplicationJson, buf)
            .await?;
        Ok(())
    }

    async fn log_request(&self, req: &Request) -> anyhow::Result<()> {
        let body = if let Some(mut body) = req.body().and_then(|b| b.as_bytes()) {
            if let Ok(json) = serde_json::from_reader(&mut body) {
                json
            } else {
                Value::String(String::from_utf8_lossy(body).to_string())
            }
        } else {
            Value::String(String::new())
        };

        let headers = req
            .headers()
            .iter()
            .map(|(k, v)| format!("{}: {}", k, String::from_utf8_lossy(v.as_bytes())))
            .collect::<Vec<String>>();

        let mut buf = Vec::new();
        for header in headers.into_iter() {
            buf.write_all(format!("{header}\n").as_bytes()).await?;
        }

        self.add_attachment("Request Headers", Mime::Txt, buf)
            .await?;

        let mut buf = Vec::new();
        match body {
            Value::String(s) => {
                buf.write_all(s.as_bytes()).await.unwrap();
                self.add_attachment("Request Body", Mime::Txt, buf).await?;
            }
            value => {
                serde_json::to_writer_pretty(&mut buf, &value).unwrap();
                self.add_attachment("Request Body", Mime::ApplicationJson, buf)
                    .await?;
            }
        }
        Ok(())
    }
}
