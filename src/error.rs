use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum DomainError {
    #[error("Invalid SKU: {0}")]
    InvalidSku(String),

    #[error("Quantity cannot be negative")]
    NegativeStock,

    #[error("Quantity overflow occurred")]
    QuantityOverflow,

    #[error("Insufficient stock. Available: {available}, Requested: {requested}")]
    InsufficientStock { available: u32, requested: u32 },

    #[error("Insufficient available stock. Available: {available}, Requested: {requested}")]
    InsufficientAvailableStock { available: u32, requested: u32 },

    #[error("Invalid reservation release")]
    InvalidReservationRelease,

    #[error("Item not found in inventory")]
    ItemNotFound,

    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },
}

#[derive(Serialize)]
pub struct ProblemDetails {
    pub r#type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail) = match self {
            AppError::Domain(DomainError::InsufficientStock { available, requested }) => (
                StatusCode::CONFLICT,
                "Insufficient Stock".to_string(),
                format!("Only {available} units available, requested {requested}."),
            ),
            AppError::Domain(DomainError::InsufficientAvailableStock { available, requested }) => (
                StatusCode::CONFLICT,
                "Insufficient Available Stock".to_string(),
                format!("Only {available} units available for reservation, requested {requested}."),
            ),
            AppError::Domain(err) => (
                StatusCode::BAD_REQUEST,
                "Domain Validation Failed".to_string(),
                err.to_string(),
            ),
            AppError::Database(err) => {
                tracing::error!("Database query failed: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error".to_string(),
                    "An unexpected error occurred while communicating with the database.".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found".to_string(), msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string(), msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "Forbidden".to_string(), msg),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "Validation Failed".to_string(), msg),
            AppError::Internal(err) => {
                tracing::error!("Internal server error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An internal server error occurred.".to_string(),
                )
            }
        };

        let problem = ProblemDetails {
            r#type: format!("https://api.ims.local/errors/{}", status.as_u16()),
            title,
            status: status.as_u16(),
            detail,
            timestamp: chrono::Utc::now(),
        };

        (status, Json(problem)).into_response()
    }
}
