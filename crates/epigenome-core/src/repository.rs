use genome_core::AssemblyAccession;

use crate::experiment::Experiment;
use crate::ids::ExperimentId;
use crate::peak::Peak;
use crate::query::{ExperimentQuery, PeakRegionQuery};

/// One peak together with the experiment it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct PeakHit {
    pub experiment_id: ExperimentId,
    pub peak: Peak,
}

/// Abstract storage layer for epigenome data.
///
/// Concrete implementations live in `epigenome-store` so this crate stays
/// I/O- and async-free, matching the `expression-core` / `expression-store`
/// split.
pub trait EpigenomeRepository: Send + Sync + 'static {
    fn experiment(&self, id: &ExperimentId) -> Option<Experiment>;

    fn experiments(&self, query: &ExperimentQuery) -> Vec<Experiment>;

    fn experiments_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Experiment>;

    /// Peaks overlapping the given half-open region, optionally filtered by
    /// assay / target / experiment-id set. Honours the query's `limit`.
    fn peaks_in_region(&self, query: &PeakRegionQuery) -> Vec<PeakHit>;

    /// All peaks belonging to one experiment, in genomic order. Used by the
    /// experiment-detail page and JBrowse track endpoint.
    fn peaks_for_experiment(&self, experiment_id: &ExperimentId) -> Vec<Peak>;
}
