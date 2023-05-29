use axum::{routing::get, Router};
use crate::handlers::{system_info_handler, run_test_handler};


pub fn create_router() -> Router {
    Router::new()
        .route("/api/system_info", get(system_info_handler))
        .route("/api/run_test", post(run_test_handler))
}
