use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::axum::handlers;
use crate::axum::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/api/health", get(handlers::health::health_check))
        // Papers
        .route("/api/papers", get(handlers::papers::list_papers))
        .route("/api/papers/{id}", get(handlers::papers::get_paper))
        // Categories
        .route(
            "/api/categories",
            get(handlers::categories::list_categories),
        )
        // Labels
        .route("/api/labels", get(handlers::labels::list_labels))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
