use epigenome_core::{Experiment, ExperimentId, Peak, PeakKind};
use genome_core::AssemblyAccession;
use serde::{Deserialize, Serialize};

/// All peaks for one experiment, grouped together so file IO stays
/// experiment-local and `peaks_for_experiment` is O(experiment) instead of
/// scanning the whole dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExperimentPeaks {
    pub experiment_id: ExperimentId,
    pub kind: PeakKind,
    #[serde(default)]
    pub peaks: Vec<Peak>,
}

/// Aggregate root for the in-memory epigenome store, scoped to one assembly.
///
/// Mirrors `ExpressionDataset`: one dataset per assembly, holding all
/// experiments and their peak calls. Multi-assembly setups should hold one
/// dataset per assembly and route queries by accession.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpigenomeDataset {
    pub assembly_accession: AssemblyAccession,
    #[serde(default)]
    pub experiments: Vec<Experiment>,
    #[serde(default)]
    pub peaks: Vec<ExperimentPeaks>,
}
