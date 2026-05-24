use genome_core::{AssemblyAccession, GeneId};

use crate::ids::{ExperimentId, SampleId};
use crate::matrix::ExpressionMatrix;
use crate::measurement::ExpressionMeasurement;
use crate::sample::{Experiment, Sample};
use crate::unit::ExpressionUnit;

/// Query parameters for looking up expression measurements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionQuery {
    pub sample_ids: Option<Vec<SampleId>>,
    pub experiment_id: Option<ExperimentId>,
    pub unit: Option<ExpressionUnit>,
    pub limit: Option<usize>,
}

/// Abstract storage layer for expression data.
///
/// Concrete implementations (e.g. a Parquet/DuckDB-backed
/// `expression-store` crate) live outside this crate so the domain types stay
/// I/O-free.
pub trait ExpressionRepository: Send + Sync + 'static {
    fn sample(&self, sample_id: &SampleId) -> Option<Sample>;
    fn samples_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sample>;
    fn samples_for_experiment(&self, experiment_id: &ExperimentId) -> Vec<Sample>;
    fn experiment(&self, experiment_id: &ExperimentId) -> Option<Experiment>;

    /// Expression values for a single gene, optionally restricted by the
    /// query parameters.
    fn gene_expression(
        &self,
        gene_id: &GeneId,
        query: &ExpressionQuery,
    ) -> Vec<ExpressionMeasurement>;

    /// Dense expression matrix for the given genes × samples in the requested
    /// unit. Returns `None` if the repository cannot satisfy the request
    /// (e.g. some genes or samples are unknown, or the unit is unsupported).
    fn expression_matrix(
        &self,
        accession: &AssemblyAccession,
        gene_ids: &[GeneId],
        sample_ids: &[SampleId],
        unit: ExpressionUnit,
    ) -> Option<ExpressionMatrix>;
}
