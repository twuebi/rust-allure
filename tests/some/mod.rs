use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub struct Server {
    pub addr: SocketAddr,
    router: Router,
}

impl Server {
    pub fn new(port: u16) -> Self {
        let app = Router::new().route("/", get(root)).route("/json", get(jsn));
        let addr = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port));
        Self { addr, router: app }
    }
    pub async fn serve(self) {
        let listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.router).await.unwrap();
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
