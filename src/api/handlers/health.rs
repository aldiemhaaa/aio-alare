use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::PgPool;

pub async fn liveness_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "UP" })))
}

pub async fn readiness_handler(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "READY",
                "database": "CONNECTED"
            })),
        ),
        Err(err) => {
            tracing::error!("Readiness check failed: {:?}", err);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "NOT_READY",
                    "database": "DISCONNECTED"
                })),
            )
        }
    }
}
