use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::quantity::StockQuantity;
use crate::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseInventory {
    pub id: Uuid,
    pub warehouse_id: Uuid,
    pub product_variant_id: Uuid,
    pub location_id: Option<Uuid>,
    pub quantity: StockQuantity,
    pub reserved_quantity: StockQuantity,
    pub min_threshold: StockQuantity,
}

impl WarehouseInventory {
    pub fn new(
        warehouse_id: Uuid,
        product_variant_id: Uuid,
        location_id: Option<Uuid>,
        min_threshold: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            warehouse_id,
            product_variant_id,
            location_id,
            quantity: StockQuantity::ZERO,
            reserved_quantity: StockQuantity::ZERO,
            min_threshold: StockQuantity::new(min_threshold),
        }
    }

    /// Kuantitas stok yang siap dijual atau dialokasikan
    pub fn available_quantity(&self) -> u32 {
        self.quantity.value().saturating_sub(self.reserved_quantity.value())
    }

    /// Mereservasi stok untuk order keluar
    pub fn reserve_stock(&mut self, amount: u32) -> Result<(), DomainError> {
        let available = self.available_quantity();
        if available < amount {
            return Err(DomainError::InsufficientAvailableStock {
                available,
                requested: amount,
            });
        }
        self.reserved_quantity = self.reserved_quantity.checked_add(amount)?;
        Ok(())
    }

    /// Melepas reservasi (misal jika order dibatalkan)
    pub fn release_reservation(&mut self, amount: u32) -> Result<(), DomainError> {
        if self.reserved_quantity.value() < amount {
            return Err(DomainError::InvalidReservationRelease);
        }
        self.reserved_quantity = self.reserved_quantity.checked_sub(amount)?;
        Ok(())
    }

    /// Mengonfirmasi pengeluaran stok (order dikirim)
    pub fn fulfill_reservation(&mut self, amount: u32) -> Result<(), DomainError> {
        self.release_reservation(amount)?;
        self.quantity = self.quantity.checked_sub(amount)?;
        Ok(())
    }

    /// Menambah stok dari penerimaan barang (Inbound)
    pub fn receive_stock(&mut self, amount: u32) -> Result<(), DomainError> {
        self.quantity = self.quantity.checked_add(amount)?;
        Ok(())
    }

    /// Mengurangi stok langsung (misal: adjustment kerusakan)
    pub fn deduct_direct(&mut self, amount: u32) -> Result<(), DomainError> {
        let available = self.available_quantity();
        if available < amount {
            return Err(DomainError::InsufficientStock {
                available,
                requested: amount,
            });
        }
        self.quantity = self.quantity.checked_sub(amount)?;
        Ok(())
    }

    /// Memeriksa apakah persediaan menyentuh ambang batas minimum
    pub fn is_low_stock(&self) -> bool {
        self.available_quantity() <= self.min_threshold.value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_lifecycle() {
        let mut inv = WarehouseInventory::new(Uuid::new_v4(), Uuid::new_v4(), None, 10);
        
        // Receive 50
        inv.receive_stock(50).unwrap();
        assert_eq!(inv.quantity.value(), 50);
        assert_eq!(inv.available_quantity(), 50);

        // Reserve 20
        inv.reserve_stock(20).unwrap();
        assert_eq!(inv.reserved_quantity.value(), 20);
        assert_eq!(inv.available_quantity(), 30);

        // Cannot reserve more than available
        let err = inv.reserve_stock(35);
        assert!(err.is_err());

        // Fulfill 20
        inv.fulfill_reservation(20).unwrap();
        assert_eq!(inv.quantity.value(), 30);
        assert_eq!(inv.reserved_quantity.value(), 0);
        assert_eq!(inv.available_quantity(), 30);
    }
}
