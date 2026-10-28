use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "movement_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MovementType {
    InboundPo,
    OutboundSales,
    TransferIn,
    TransferOut,
    AdjustmentAdd,
    AdjustmentSub,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    PurchaseOrder,
    SalesOrder,
    StockOpname,
    WarehouseTransfer,
}

impl ReferenceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PurchaseOrder => "PURCHASE_ORDER",
            Self::SalesOrder => "SALES_ORDER",
            Self::StockOpname => "STOCK_OPNAME",
            Self::WarehouseTransfer => "WAREHOUSE_TRANSFER",
        }
    }
}
