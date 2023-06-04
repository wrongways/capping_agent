use axum::{routing::{get, post}, Router};
use crate::handlers::{
    system_info_handler::system_info_handler,
    run_test_handler::run_test_handler,
    fallback_handler::fallback
};


pub fn create_router() -> Router {
    Router::new()
        .route("/api/system_info", get(system_info_handler))
        .route("/api/run_test", post(run_test_handler))
        .fallback(fallback)
}
