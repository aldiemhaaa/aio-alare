use serde::{Deserialize, Serialize};
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct StockQuantity(u32);

impl StockQuantity {
    pub const ZERO: Self = Self(0);

    pub fn new(val: u32) -> Self {
        Self(val)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn checked_add(self, delta: u32) -> Result<Self, DomainError> {
        self.0
            .checked_add(delta)
            .map(Self)
            .ok_or(DomainError::QuantityOverflow)
    }

    pub fn checked_sub(self, delta: u32) -> Result<Self, DomainError> {
        self.0
            .checked_sub(delta)
            .map(Self)
            .ok_or(DomainError::NegativeStock)
    }
}

impl std::fmt::Display for StockQuantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for StockQuantity {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_subtraction_success() {
        let q = StockQuantity::new(10);
        let updated = q.checked_sub(4).unwrap();
        assert_eq!(updated.value(), 6);
    }

    #[test]
    fn stock_subtraction_prevent_negative() {
        let q = StockQuantity::new(5);
        let result = q.checked_sub(6);
        assert!(result.is_err());
    }
}
