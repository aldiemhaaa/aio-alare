use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::application::dtos::{AdjustStockRequest, InventorySummaryDto, StockMovementDto};
use crate::domain::MovementType;
use crate::error::AppError;

#[derive(Debug, FromRow)]
struct InventoryLockRow {
    id: Uuid,
    quantity: i32,
    #[allow(dead_code)]
    reserved_quantity: i32,
}

#[derive(Debug, FromRow)]
struct InventoryRecordRow {
    warehouse_id: Uuid,
    warehouse_name: String,
    product_variant_id: Uuid,
    sku: String,
    variant_name: String,
    quantity: i32,
    reserved_quantity: i32,
    available_quantity: Option<i32>,
    min_threshold: i32,
}

pub struct InventoryService {
    pool: PgPool,
}

impl InventoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Melakukan penyesuaian stok (Stock Opname / Adjustment) dengan Row-Level Lock
    pub async fn adjust_stock(
        &self,
        req: AdjustStockRequest,
        user_id: Option<Uuid>,
    ) -> Result<StockMovementDto, AppError> {
        let mut tx = self.pool.begin().await?;

        // 1. Kunci row inventaris (Pessimistic Lock)
        let row: Option<InventoryLockRow> = sqlx::query_as(
            r#"
            SELECT id, quantity, reserved_quantity
            FROM inventories
            WHERE warehouse_id = $1 AND product_variant_id = $2
            FOR UPDATE
            "#
        )
        .bind(req.warehouse_id)
        .bind(req.product_variant_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (inventory_id, current_qty) = match row {
            Some(r) => (r.id, r.quantity),
            None => {
                let new_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO inventories (id, warehouse_id, product_variant_id, quantity, reserved_quantity)
                    VALUES ($1, $2, $3, 0, 0)
                    "#
                )
                .bind(new_id)
                .bind(req.warehouse_id)
                .bind(req.product_variant_id)
                .execute(&mut *tx)
                .await?;

                (new_id, 0)
            }
        };

        let new_qty = req.actual_quantity as i32;
        let delta = new_qty - current_qty;

        if delta == 0 {
            tx.rollback().await?;
            return Err(AppError::Validation("No quantity change detected".to_string()));
        }

        let movement_type = if delta > 0 {
            MovementType::AdjustmentAdd
        } else {
            MovementType::AdjustmentSub
        };

        // 2. Perbarui jumlah stok di inventaris
        sqlx::query(
            r#"
            UPDATE inventories
            SET quantity = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(new_qty)
        .bind(inventory_id)
        .execute(&mut *tx)
        .await?;

        // 3. Catat entri audit ledger di stock_movements
        let movement_id = Uuid::new_v4();
        let notes_text = req.notes.unwrap_or_else(|| req.reason.clone());

        sqlx::query(
            r#"
            INSERT INTO stock_movements (
                id, warehouse_id, product_variant_id, movement_type,
                quantity_delta, balance_after, reference_type, notes, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'STOCK_OPNAME', $7, $8)
            "#
        )
        .bind(movement_id)
        .bind(req.warehouse_id)
        .bind(req.product_variant_id)
        .bind(movement_type)
        .bind(delta)
        .bind(new_qty)
        .bind(&notes_text)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // 4. Commit transaksi
        tx.commit().await?;

        Ok(StockMovementDto {
            id: movement_id,
            warehouse_id: req.warehouse_id,
            product_variant_id: req.product_variant_id,
            movement_type: format!("{:?}", movement_type),
            quantity_delta: delta,
            balance_after: new_qty,
            reference_type: "STOCK_OPNAME".to_string(),
            notes: Some(notes_text),
            created_at: chrono::Utc::now(),
        })
    }

    /// Mengambil daftar ringkasan stok gudang
    pub async fn get_inventory_list(
        &self,
        warehouse_id: Option<Uuid>,
        low_stock_only: bool,
    ) -> Result<Vec<InventorySummaryDto>, AppError> {
        let records: Vec<InventoryRecordRow> = sqlx::query_as(
            r#"
            SELECT 
                i.warehouse_id,
                w.name AS warehouse_name,
                i.product_variant_id,
                pv.sku,
                pv.variant_name,
                i.quantity,
                i.reserved_quantity,
                (i.quantity - i.reserved_quantity) AS available_quantity,
                i.min_threshold
            FROM inventories i
            JOIN warehouses w ON w.id = i.warehouse_id
            JOIN product_variants pv ON pv.id = i.product_variant_id
            WHERE ($1::uuid IS NULL OR i.warehouse_id = $1)
              AND (NOT $2 OR (i.quantity - i.reserved_quantity) <= i.min_threshold)
            ORDER BY w.name, pv.sku
            "#
        )
        .bind(warehouse_id)
        .bind(low_stock_only)
        .fetch_all(&self.pool)
        .await?;

        let items = records
            .into_iter()
            .map(|r| {
                let available = r.available_quantity.unwrap_or(0);
                InventorySummaryDto {
                    warehouse_id: r.warehouse_id,
                    warehouse_name: r.warehouse_name,
                    product_variant_id: r.product_variant_id,
                    sku: r.sku,
                    variant_name: r.variant_name,
                    quantity_on_hand: r.quantity,
                    reserved_quantity: r.reserved_quantity,
                    available_quantity: available,
                    min_threshold: r.min_threshold,
                    is_low_stock: available <= r.min_threshold,
                }
            })
            .collect();

        Ok(items)
    }
}
