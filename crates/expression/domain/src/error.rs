#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ExpressionError {
    #[error("invalid SRA Run accession: {0}")]
    InvalidSraRunAccession(String),
    #[error("invalid SRA Experiment accession: {0}")]
    InvalidSraExperimentAccession(String),
    #[error("invalid SRA Study accession: {0}")]
    InvalidSraStudyAccession(String),
    #[error("invalid BioSample accession: {0}")]
    InvalidBioSampleAccession(String),
    #[error("invalid BioProject accession: {0}")]
    InvalidBioProjectAccession(String),
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
