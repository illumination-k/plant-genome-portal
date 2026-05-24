use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("domain error: {0}")]
    Domain(#[from] genome_core::DomainError),
    #[error("invalid gff record on line {line}: {message}")]
    InvalidGffValue { line: usize, message: String },
    #[error("invalid tsv line {line}: {message}")]
    InvalidTsvValue { line: usize, message: String },
    #[error("missing gff attribute {attribute} on line {line}")]
    MissingGffAttribute {
        line: usize,
        attribute: &'static str,
    },
    #[error("missing FASTA sequence for {0}")]
    MissingFastaSequence(String),
    #[error("invalid FASTA record: {0}")]
    InvalidFastaRecord(PathBuf),
}
