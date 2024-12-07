use askama_axum::Template;
use chrono::Utc;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}

fn router() -> Router {
    let serve_dir = ServeDir::new("assets");
    Router::new()
        .route("/", get(index))
        .fallback_service(serve_dir)
}

async fn index() -> impl IntoResponse {
    IndexTemplate {
        date: Utc::now().to_rfc3339(),
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    date: String,
}
