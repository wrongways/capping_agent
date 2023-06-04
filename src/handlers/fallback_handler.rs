use axum::http::{StatusCode, Uri};
use log::error;

pub async fn fallback(uri: Uri) -> (StatusCode, String) {
    error!("Request for unknown URI: {uri}");
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}
