#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("{kind} must not be empty")]
    EmptyIdentifier { kind: &'static str },
    #[error("invalid GO term id: {0}")]
    InvalidGoTermId(String),
    #[error("invalid InterPro id: {0}")]
    InvalidInterProId(String),
    #[error("invalid Pfam accession: {0}")]
    InvalidPfamAccession(String),
    #[error("1-based positions must be greater than zero")]
    ZeroPosition1,
    #[error("region start must be <= end")]
    InvalidClosedRegion,
    #[error("region start must be < end")]
    InvalidHalfOpenRegion,
    #[error("invalid strand: {0}")]
    InvalidStrand(String),
    #[error("invalid region expression: {0}")]
    InvalidRegionExpression(String),
}
