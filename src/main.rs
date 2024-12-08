use askama_axum::Template;
use chrono::Utc;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{response::IntoResponse, routing::get, Router};
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
        .route("/current_datetime", get(current_datetime))
        .fallback_service(serve_dir)
}

async fn index() -> impl IntoResponse {
    IndexTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

async fn current_datetime() -> impl IntoResponse {
    CurrentDateTimeTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    datetime: String,
}

#[derive(Template)]
#[template(path = "current_datetime.html")]
struct CurrentDateTimeTemplate {
    datetime: String,
}
