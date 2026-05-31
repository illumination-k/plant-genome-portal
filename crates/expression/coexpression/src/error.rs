#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoexpressionError {
    #[error("vector length mismatch: left has {left} values, right has {right}")]
    VectorLengthMismatch { left: usize, right: usize },
    #[error("expression matrix dimension mismatch: expected {expected} values, got {actual}")]
    ExpressionMatrixDimensionMismatch { expected: usize, actual: usize },
    #[error("expression matrix shape is invalid")]
    InvalidMatrixShape,
    #[error("matrix is too large to index")]
    MatrixTooLarge,
    #[error("matrix layout is not contiguous row-major")]
    MatrixLayout,
    #[error("cannot compute correlation for an empty expression matrix")]
    EmptyCorrelationInput,
    #[error("too many genes to store directional ranks as u32: {gene_count}")]
    TooManyGenesForRank { gene_count: usize },
}
