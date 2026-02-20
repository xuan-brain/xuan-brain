use std::net::SocketAddr;
use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tauri::AppHandle;
use tracing::info;

use crate::axum::routes::create_router;
use crate::axum::state::AppState;
use crate::sys::dirs::AppDirs;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3030;

pub fn start_axum_server(db: Arc<DatabaseConnection>, app_dirs: AppDirs) {
    let addr: SocketAddr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT)
        .parse()
        .expect("Invalid API server address");

    let state = AppState::new(db, app_dirs);
    let app = create_router(state);

    info!("Starting Axum API server on {}", addr);
    info!("Swagger UI available at http://{}/swagger-ui/", addr);

    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind Axum server: {}", e);
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Axum server error: {}", e);
        }
    });
}

pub fn start_axum_server_with_handle(
    db: Arc<DatabaseConnection>,
    app_dirs: AppDirs,
    app_handle: AppHandle,
) {
    let addr: SocketAddr = format!("{}:{}", DEFAULT_HOST, DEFAULT_PORT)
        .parse()
        .expect("Invalid API server address");

    let state = AppState::new_with_handle(db, app_dirs, app_handle);
    let app = create_router(state);

    info!("Starting Axum API server on {}", addr);
    info!("Swagger UI available at http://{}/swagger-ui/", addr);

    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind Axum server: {}", e);
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("Axum server error: {}", e);
        }
    });
}
