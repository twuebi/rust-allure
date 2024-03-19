use crate::{Attachment, TestResultBuilder};
use http::HeaderMap;
use reqwest::{Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use task_local_extensions::Extensions;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub struct LoggingMiddleware {
    allure_dir: PathBuf,
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        self.log_request(&req, extensions).await;

        let res = next.run(req, extensions).await?;
        let (body, headers, response) = Self::prepare_response_copy(res).await?;

        self.log_response(extensions, headers, body).await?;

        Ok(response.into())
    }
}

impl LoggingMiddleware {
    pub fn new(allure_dir: PathBuf) -> Self {
        if !allure_dir.exists() {
            std::fs::create_dir(&allure_dir).unwrap();
        }
        Self { allure_dir }
    }

    async fn add_attachment(
        &self,
        name: &str,
        mime: Mime,
        content: Vec<u8>,
        extensions: &mut Extensions,
    ) {
        let of_name = format!("{}-attachment.{}", Uuid::new_v4(), mime.to_ext())
            .try_into()
            .unwrap();
        let mut of = self.allure_dir.clone();
        of.push(&of_name);

        tokio::fs::write(of, &content).await.unwrap();

        {
            let ext: Option<&mut TestResultBuilder> = extensions.get_mut();
            if let Some(ext) = ext {
                ext.current_step().map(|step| {
                    step.attachments.push(Attachment {
                        name: name.to_string(),
                        source: of_name,
                        r#type: mime.to_string(),
                    })
                });
            };
        }
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

    async fn log_response(
        &self,
        extensions: &mut Extensions,
        headers: HeaderMap,
        body: bytes::Bytes,
    ) -> Result<()> {
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
            buf.write(&format!("{header}\n").as_bytes()).await.unwrap();
        }

        self.add_attachment("Response Headers", Mime::Txt, buf, extensions)
            .await;

        let mut buf = Vec::new();
        serde_json::to_writer_pretty(&mut buf, &body_v).unwrap();
        self.add_attachment("Response Body", Mime::ApplicationJson, buf, extensions)
            .await;
        Ok(())
    }

    async fn log_request(&self, req: &Request, extensions: &mut Extensions) {
        let body = if let Some(mut body) = req.body().as_deref().map(|b| b.as_bytes()).flatten() {
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
            buf.write(&format!("{header}\n").as_bytes()).await.unwrap();
        }

        self.add_attachment("Request Headers", Mime::Txt, buf, extensions)
            .await;

        let mut buf = Vec::new();
        match body {
            Value::String(s) => {
                buf.write(s.as_bytes()).await.unwrap();
                self.add_attachment("Request Body", Mime::Txt, buf, extensions)
                    .await;
            }
            value => {
                serde_json::to_writer_pretty(&mut buf, &value).unwrap();
                self.add_attachment("Request Body", Mime::ApplicationJson, buf, extensions)
                    .await;
            }
        }
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
    fn to_ext(&self) -> &'static str {
        match self {
            Mime::ApplicationJson => "json",
            Mime::Txt => "txt",
        }
    }
}
