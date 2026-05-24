use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::ExpressionError;
use crate::unit::ExpressionUnit;

/// A validated expression measurement value paired with its unit.
///
/// The value is required to be finite (no `NaN`, no infinities) and
/// non-negative. Count units (`RawCount`, `NormalizedCount`) are additionally
/// required to be integer-valued in the [`RawCount`] case.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ExpressionValue {
    pub value: f64,
    pub unit: ExpressionUnit,
}

impl ExpressionValue {
    pub fn new(value: f64, unit: ExpressionUnit) -> Result<Self, ExpressionError> {
        if !value.is_finite() || value < 0.0 {
            return Err(ExpressionError::InvalidExpressionValue(value));
        }
        if unit == ExpressionUnit::RawCount && value.fract() != 0.0 {
            return Err(ExpressionError::InvalidExpressionValue(value));
        }
        Ok(Self { value, unit })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_and_positive_finite_values() {
        assert!(ExpressionValue::new(0.0, ExpressionUnit::Tpm).is_ok());
        assert!(ExpressionValue::new(12.34, ExpressionUnit::Fpkm).is_ok());
        assert!(ExpressionValue::new(0.0, ExpressionUnit::RawCount).is_ok());
        assert!(ExpressionValue::new(42.0, ExpressionUnit::RawCount).is_ok());
    }

    #[test]
    fn rejects_negative_values() {
        assert!(matches!(
            ExpressionValue::new(-0.1, ExpressionUnit::Tpm),
            Err(ExpressionError::InvalidExpressionValue(_))
        ));
    }

    #[test]
    fn rejects_non_finite_values() {
        assert!(ExpressionValue::new(f64::NAN, ExpressionUnit::Tpm).is_err());
        assert!(ExpressionValue::new(f64::INFINITY, ExpressionUnit::Tpm).is_err());
        assert!(ExpressionValue::new(f64::NEG_INFINITY, ExpressionUnit::Tpm).is_err());
    }

    #[test]
    fn raw_count_must_be_integer_valued() {
        assert!(ExpressionValue::new(3.5, ExpressionUnit::RawCount).is_err());
        // NormalizedCount permits fractional values (e.g. DESeq2 scaling).
        assert!(ExpressionValue::new(3.5, ExpressionUnit::NormalizedCount).is_ok());
    }
}
