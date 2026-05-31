use genome_domain::{AssemblyAccession, HalfOpenRegion};

use crate::assay::Assay;
use crate::ids::{ExperimentId, Target};

/// Filters for `EpigenomeRepository::experiments`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExperimentQuery {
    pub assembly_accession: Option<AssemblyAccession>,
    pub assay: Option<Assay>,
    pub target: Option<Target>,
    pub limit: Option<usize>,
}

/// Filters for `EpigenomeRepository::peaks_in_region`.
///
/// `region` is the *half-open* internal representation; API handlers convert
/// from `ClosedRegion` (1-based closed) before calling the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeakRegionQuery {
    pub assembly_accession: AssemblyAccession,
    pub region: HalfOpenRegion,
    pub experiments: Option<Vec<ExperimentId>>,
    pub assay: Option<Assay>,
    pub target: Option<Target>,
    pub limit: Option<usize>,
}
