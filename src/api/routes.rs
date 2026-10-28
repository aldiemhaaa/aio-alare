use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::api::handlers::{health, inventory};

pub fn create_router(pool: PgPool) -> Router {
    let api_v1 = Router::new()
        // Inventory endpoints
        .route("/inventory", get(inventory::list_inventory_handler))
        .route("/inventory/adjust", post(inventory::adjust_stock_handler));

    Router::new()
        // Health check endpoints
        .route("/health/live", get(health::liveness_handler))
        .route("/health/ready", get(health::readiness_handler))
        // API v1 routes
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(pool)
}
