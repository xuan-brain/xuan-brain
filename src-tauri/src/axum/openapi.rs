use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::axum::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::papers::list_papers,
        handlers::papers::get_paper,
        handlers::papers::import_paper_from_html,
        handlers::categories::list_categories,
        handlers::labels::list_labels,
        handlers::clips::create_clip,
        handlers::clips::list_clips,
        handlers::clips::get_clip,
    ),
    components(schemas(
        handlers::papers::ImportHtmlResponse,
        handlers::clips::CreateClippingRequest,
        handlers::clips::CreateClippingResponse,
        handlers::clips::ClippingResponse,
        handlers::clips::ListClipsQuery,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "papers", description = "Paper management endpoints"),
        (name = "categories", description = "Category management endpoints"),
        (name = "labels", description = "Label management endpoints"),
        (name = "clips", description = "Web clipping management endpoints"),
    ),
    info(
        title = "Xuan Brain API",
        version = "0.1.0",
        description = "API for Xuan Brain - AI-powered research paper management system",
    )
)]
pub struct ApiDoc;

pub fn create_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
