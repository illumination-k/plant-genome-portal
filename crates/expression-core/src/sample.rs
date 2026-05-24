use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use genome_core::AssemblyAccession;

use crate::ids::{ExperimentId, SampleId};

/// Metadata for a single RNA-seq (or similar) sample.
///
/// Free-form fields (`tissue`, `developmental_stage`, ...) are intentionally
/// `Option<String>` rather than typed enums — plant ontologies are diverse
/// (PO, PECO, EFO, ENVO) and locking in a vocabulary at this layer would
/// preclude downstream use cases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Sample {
    pub id: SampleId,
    pub assembly_accession: AssemblyAccession,
    pub experiment_id: Option<ExperimentId>,
    pub title: Option<String>,
    pub tissue: Option<String>,
    pub developmental_stage: Option<String>,
    pub treatment: Option<String>,
    pub condition: Option<String>,
    pub replicate: Option<u32>,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

/// Metadata for an experiment grouping one or more samples.
///
/// `external_accession` typically holds the SRA / ENA BioProject, GEO Series,
/// or ArrayExpress identifier the experiment was sourced from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Experiment {
    pub id: ExperimentId,
    pub title: String,
    pub description: Option<String>,
    pub external_accession: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}
