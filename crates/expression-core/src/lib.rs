//! Core expression-level domain types.
//!
//! This crate intentionally has no I/O or async dependencies — it only
//! contains the identifiers, value types, and repository trait used by
//! higher layers (storage, service, API) to talk about gene expression.
//!
//! Identifiers are based on the SRA / INSDC hierarchy: an expression
//! [`Sample`] is keyed by an [`SraRunAccession`] (one Run = one FASTQ pair =
//! one quantification) and carries its parent Experiment / Study /
//! BioSample / BioProject accessions.

mod error;
mod ids;
mod matrix;
mod measurement;
mod repository;
mod sample;
mod unit;
mod value;

pub use error::ExpressionError;
pub use ids::{
    BioProjectAccession, BioSampleAccession, SraExperimentAccession, SraRunAccession,
    SraStudyAccession,
};
pub use matrix::ExpressionMatrix;
pub use measurement::ExpressionMeasurement;
pub use repository::{ExpressionQuery, ExpressionRepository};
pub use sample::{BioProject, Sample};
pub use unit::ExpressionUnit;
pub use value::ExpressionValue;
