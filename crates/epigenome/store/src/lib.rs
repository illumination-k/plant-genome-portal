//! In-memory epigenome (ChIP-seq + ATAC-seq) store.
//!
//! Mirrors the genome-store and expression-store crates:
//! holds an [`EpigenomeDataset`] in memory, round-trips it through a JSON
//! snapshot, and provides [`FileEpigenomeRepository`] — an implementation of
//! [`epigenome_domain::EpigenomeRepository`] backed by that dataset.
//!
//! The MACS narrowPeak / broadPeak parsers and the curator TOML manifest
//! parser live in [`parsers`] and are used by `portal-cli import
//! epigenome-manifest` to build the snapshot from on-disk peak files.

mod dataset;
mod error;
pub mod parsers;
mod repository;
mod snapshot;

pub use crate::dataset::{EpigenomeDataset, ExperimentPeaks};
pub use crate::error::EpigenomeStoreError;
pub use crate::repository::FileEpigenomeRepository;
pub use crate::snapshot::{
    EpigenomeSnapshot, EpigenomeSnapshotManifest, read_snapshot, write_snapshot,
};
