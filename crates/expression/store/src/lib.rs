//! In-memory expression store.
//!
//! Mirrors the genome-store crate: holds an
//! [`ExpressionDataset`] in memory, round-trips it through a JSON snapshot,
//! and provides [`FileExpressionRepository`] — an implementation of
//! [`expression_domain::ExpressionRepository`] backed by that dataset.
//!
//! A Parquet/DuckDB-backed store can replace this once expression data
//! outgrows the single-snapshot model.

mod dataset;
mod error;
mod repository;
mod snapshot;

pub use crate::dataset::ExpressionDataset;
pub use crate::error::ExpressionStoreError;
pub use crate::repository::FileExpressionRepository;
pub use crate::snapshot::{
    ExpressionSnapshot, ExpressionSnapshotManifest, read_snapshot, write_snapshot,
};
