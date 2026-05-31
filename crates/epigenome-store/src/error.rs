use epigenome_core::{EpigenomeError, ExperimentId};
use expression_core::ExpressionError;
use genome_core::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum EpigenomeStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("epigenome domain error: {0}")]
    Epigenome(#[from] EpigenomeError),
    #[error("expression-core domain error: {0}")]
    Expression(#[from] ExpressionError),
    #[error("genome-core domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("duplicate experiment id: {0}")]
    DuplicateExperiment(ExperimentId),
    #[error(
        "experiment {experiment} has assembly {experiment_assembly} but dataset is {dataset_assembly}"
    )]
    ExperimentAssemblyMismatch {
        experiment: ExperimentId,
        experiment_assembly: String,
        dataset_assembly: String,
    },
    #[error("peaks reference unknown experiment id: {0}")]
    UnknownExperimentInPeaks(ExperimentId),
    #[error("invalid {format} line {line}: {reason}")]
    InvalidPeakLine {
        format: &'static str,
        line: usize,
        reason: String,
    },
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
}
