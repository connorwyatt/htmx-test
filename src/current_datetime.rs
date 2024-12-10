use askama::Template;
use askama_axum::IntoResponse;
use axum::{routing::get, Router};
use chrono::Utc;

use crate::AppState;

pub(crate) fn current_datetime_router() -> Router<AppState> {
    Router::new()
        .route("/", get(current_datetime))
        .route("/block", get(current_datetime_block))
}

async fn current_datetime() -> impl IntoResponse {
    CurrentDateTimeTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

async fn current_datetime_block() -> impl IntoResponse {
    CurrentDateTimeBlockTemplate {
        datetime: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

#[derive(Template)]
#[template(path = "current_datetime.html")]
struct CurrentDateTimeTemplate {
    datetime: String,
}

#[derive(Template)]
#[template(path = "current_datetime_block.html")]
struct CurrentDateTimeBlockTemplate {
    datetime: String,
}
