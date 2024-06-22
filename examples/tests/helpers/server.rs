use allure_report::prelude::*;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub struct Server {
    pub addr: SocketAddr,
    router: Router,
    listener: tokio::net::TcpListener,
}

impl Server {
    pub async fn new(port: u16) -> Self {
        let app = Router::new()
            .route("/", get(root))
            .route("/json", post(jsn));
        let addr = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        Self {
            addr: listener.local_addr().unwrap(),
            router: app,
            listener,
        }
    }
    pub async fn serve(self) {
        tracing::info!("listening on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router).await.unwrap();
    }

    pub fn spawn_serve(self) {
        tokio::task::spawn(async {
            self.serve().await;
        });
    }
}

async fn root() -> impl IntoResponse {
    "Hello, World!"
}
#[derive(Deserialize, Serialize)]
pub struct Test {
    pub a: String,
}

async fn jsn(Json(_val): Json<Test>) -> impl IntoResponse {
    Json(serde_json::json!({"a": "b"}))
}
