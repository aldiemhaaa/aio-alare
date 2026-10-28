use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            meta: None,
        }
    }

    pub fn with_meta(data: T, meta: PaginationMeta) -> Self {
        Self {
            success: true,
            data,
            meta: Some(meta),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u64,
    pub limit: u64,
    pub total_records: u64,
    pub total_pages: u64,
}

#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub warehouse_id: Uuid,
    pub product_variant_id: Uuid,
    pub actual_quantity: u32,
    pub reason: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransferStockItem {
    pub product_variant_id: Uuid,
    pub quantity: u32,
}

#[derive(Debug, Deserialize)]
pub struct TransferStockRequest {
    pub source_warehouse_id: Uuid,
    pub destination_warehouse_id: Uuid,
    pub items: Vec<TransferStockItem>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventorySummaryDto {
    pub warehouse_id: Uuid,
    pub warehouse_name: String,
    pub product_variant_id: Uuid,
    pub sku: String,
    pub variant_name: String,
    pub quantity_on_hand: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
    pub min_threshold: i32,
    pub is_low_stock: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockMovementDto {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub product_variant_id: Uuid,
    pub movement_type: String,
    pub quantity_delta: i32,
    pub balance_after: i32,
    pub reference_type: String,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
