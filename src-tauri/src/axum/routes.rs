use axum::{routing::get, routing::post, Router};
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::axum::handlers;
use crate::axum::openapi::create_swagger_ui;
use crate::axum::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve static image files from clips directory
    let clips_images_dir: PathBuf = PathBuf::from(&state.app_dirs.files).join("clips");
    let serve_images = ServeDir::new(clips_images_dir.clone());

    Router::new()
        // Static file serving
        .nest_service("/clips/images", serve_images)
        // Health check
        .route("/api/health", get(handlers::health::health_check))
        // Clips
        .route("/api/clips", get(handlers::clips::list_clips))
        .route("/api/clips/{id}", get(handlers::clips::get_clip))
        .route("/api/clips", post(handlers::clips::create_clip))
        // Papers
        .route("/api/papers", get(handlers::papers::list_papers))
        .route("/api/papers/{id}", get(handlers::papers::get_paper))
        .route(
            "/api/papers/import-html",
            post(handlers::papers::import_paper_from_html),
        )
        // Categories
        .route(
            "/api/categories",
            get(handlers::categories::list_categories),
        )
        // Labels
        .route("/api/labels", get(handlers::labels::list_labels))
        // Swagger UI (always available for debugging)
        .merge(create_swagger_ui())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
