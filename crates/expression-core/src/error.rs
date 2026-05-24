#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExpressionError {
    #[error("{kind} must not be empty")]
    EmptyIdentifier { kind: &'static str },
    #[error("expression value must be finite and non-negative: {0}")]
    InvalidExpressionValue(f64),
    #[error("expression matrix dimension mismatch: expected {expected} values, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("expression matrix unit mismatch: matrix is {matrix:?}, value is {value:?}")]
    UnitMismatch {
        matrix: crate::unit::ExpressionUnit,
        value: crate::unit::ExpressionUnit,
    },
}
