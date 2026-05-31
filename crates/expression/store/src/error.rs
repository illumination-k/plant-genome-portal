use expression_domain::{ExpressionError, ExpressionUnit};

#[derive(Debug, thiserror::Error)]
pub enum ExpressionStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("expression error: {0}")]
    Expression(#[from] ExpressionError),
    #[error("duplicate expression matrix for unit {0:?}")]
    DuplicateMatrix(ExpressionUnit),
    #[error("matrix assembly {matrix} does not match dataset assembly {dataset}")]
    MatrixAssemblyMismatch { matrix: String, dataset: String },
}
