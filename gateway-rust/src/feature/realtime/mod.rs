pub mod presence;

use crate::AppState;
use axum::extract::{Query, State};
use axum::response::Sse;
use axum::response::sse::Event;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::Stream;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterPublicSse {
    #[validate(length(min = 1))]
    pub device_id: String,
    #[validate(length(min = 1))]
    pub user_id: String,
}

use axum::http::HeaderMap;

pub async fn register_public_sse(
    state: State<AppState>,
    query: Query<RegisterPublicSse>,
) -> (HeaderMap, Sse<impl Stream<Item = Result<Event, Infallible>>>) {
    let mut headers = HeaderMap::new();
    headers.insert("X-Accel-Buffering", "no".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());

    let sse = state
        .sse
        .new_client(query.user_id.clone(), query.device_id.clone())
        .await;

    (headers, sse)
}
