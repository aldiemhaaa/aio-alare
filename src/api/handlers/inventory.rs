use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::dtos::{AdjustStockRequest, ApiResponse, InventorySummaryDto, StockMovementDto};
use crate::application::InventoryService;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct InventoryQuery {
    pub warehouse_id: Option<Uuid>,
    #[serde(default)]
    pub low_stock_only: bool,
}

pub async fn list_inventory_handler(
    State(pool): State<PgPool>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<ApiResponse<Vec<InventorySummaryDto>>>, AppError> {
    let service = InventoryService::new(pool);
    let items = service
        .get_inventory_list(query.warehouse_id, query.low_stock_only)
        .await?;

    Ok(Json(ApiResponse::success(items)))
}

pub async fn adjust_stock_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<AdjustStockRequest>,
) -> Result<Json<ApiResponse<StockMovementDto>>, AppError> {
    let service = InventoryService::new(pool);
    // User ID dapat diambil dari Auth middleware di implementasi lanjutan
    let movement = service.adjust_stock(payload, None).await?;

    Ok(Json(ApiResponse::success(movement)))
}
