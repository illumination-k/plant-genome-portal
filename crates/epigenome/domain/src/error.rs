#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EpigenomeError {
    #[error("invalid GEO Series accession: {0}")]
    InvalidGeoSeriesAccession(String),
    #[error("invalid GEO Sample accession: {0}")]
    InvalidGeoSampleAccession(String),
    #[error("{kind} must not be empty")]
    EmptyIdentifier { kind: &'static str },
    #[error("unknown assay: {0}")]
    UnknownAssay(String),
}
