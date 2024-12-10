use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;

pub(crate) fn paginated_list_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_paginated_list))
        .route("/items", get(get_paginated_list_items))
}

#[derive(Deserialize)]
struct PaginationQueryParameters {
    offset: Option<usize>,
}

const PAGE_SIZE: usize = 50;

async fn get_paginated_list() -> impl IntoResponse {
    let ids: Vec<_> = (0..PAGE_SIZE).map(|_| Uuid::new_v4().to_string()).collect();
    PaginatedListTemplate {
        offset: Some(ids.len()),
        ids,
    }
}

async fn get_paginated_list_items(
    Query(query_params): Query<PaginationQueryParameters>,
) -> impl IntoResponse {
    let ids: Vec<_> = (0..PAGE_SIZE).map(|_| Uuid::new_v4().to_string()).collect();
    let offset = query_params.offset.unwrap_or(0);
    let new_offset = offset + ids.len();
    PaginatedListItemsTemplate {
        offset: if new_offset < 500 {
            Some(new_offset)
        } else {
            None
        },
        ids,
    }
}

#[derive(Template)]
#[template(path = "paginated_list.html")]
struct PaginatedListTemplate {
    ids: Vec<String>,
    offset: Option<usize>,
}

#[derive(Template)]
#[template(path = "paginated_list_items.html")]
struct PaginatedListItemsTemplate {
    ids: Vec<String>,
    offset: Option<usize>,
}
