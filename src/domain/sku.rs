use serde::{Deserialize, Serialize};
use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sku(String);

impl Sku {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let val = raw.into().trim().to_uppercase();
        if val.len() < 3 || val.len() > 50 {
            return Err(DomainError::InvalidSku(
                "SKU length must be between 3 and 50 characters".to_string(),
            ));
        }
        if !val.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(DomainError::InvalidSku(
                "SKU must contain only alphanumeric characters and hyphens".to_string(),
            ));
        }
        Ok(Self(val))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Sku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_sku_creation() {
        let sku = Sku::new("sku-item-01").unwrap();
        assert_eq!(sku.as_str(), "SKU-ITEM-01");
    }

    #[test]
    fn invalid_sku_characters() {
        let result = Sku::new("sku_item!@#");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_sku_too_short() {
        let result = Sku::new("ab");
        assert!(result.is_err());
    }
}
