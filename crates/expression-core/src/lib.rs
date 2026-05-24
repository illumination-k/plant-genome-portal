//! Core expression-level domain types.
//!
//! This crate intentionally has no I/O or async dependencies — it only
//! contains the identifiers, value types, and repository trait used by
//! higher layers (storage, service, API) to talk about gene expression.

mod error;
mod ids;
mod matrix;
mod measurement;
mod repository;
mod sample;
mod unit;
mod value;

pub use error::ExpressionError;
pub use ids::{ExperimentId, SampleId};
pub use matrix::ExpressionMatrix;
pub use measurement::ExpressionMeasurement;
pub use repository::{ExpressionQuery, ExpressionRepository};
pub use sample::{Experiment, Sample};
pub use unit::ExpressionUnit;
pub use value::ExpressionValue;
