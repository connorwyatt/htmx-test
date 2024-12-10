use askama_axum::Template;
use current_datetime::current_datetime_router;
use people::{people_router, PeopleState};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub(crate) mod current_datetime;
pub(crate) mod people;

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
    axum::serve(
        listener,
        router().with_state(AppState {
            people_state: Arc::new(RwLock::new(PeopleState::default())),
        }),
    )
    .await
    .unwrap();
}

#[derive(Clone)]
struct AppState {
    people_state: Arc<RwLock<PeopleState>>,
}

fn router() -> Router<AppState> {
    let serve_dir = ServeDir::new("assets");
    Router::new()
        .route("/", get(index))
        .nest("/current_datetime", current_datetime_router())
        .nest("/people", people_router())
        .fallback_service(serve_dir)
}

// Home

async fn index() -> impl IntoResponse {
    IndexTemplate
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;
