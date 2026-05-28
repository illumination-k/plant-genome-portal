//! Core epigenome-layer domain types.
//!
//! This crate has no I/O or async dependencies — it only contains the
//! identifiers, value types, and repository trait used by higher layers
//! (`epigenome-store`, `service`, `api`) to talk about ChIP-seq and ATAC-seq
//! data.
//!
//! The split mirrors `expression-core` / `expression-store` exactly:
//! domain types live here, file/database-backed implementations live next
//! door.

mod assay;
mod error;
mod experiment;
mod ids;
mod peak;
mod peak_kind;
mod qc;
mod query;
mod repository;

pub use assay::Assay;
pub use error::EpigenomeError;
pub use experiment::Experiment;
pub use ids::{Antibody, ExperimentId, GeoSampleAccession, GeoSeriesAccession, Target};
pub use peak::Peak;
pub use peak_kind::PeakKind;
pub use qc::ExperimentQc;
pub use query::{ExperimentQuery, PeakRegionQuery};
pub use repository::{EpigenomeRepository, PeakHit};
