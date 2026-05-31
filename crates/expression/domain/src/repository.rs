use genome_domain::{AssemblyAccession, GeneId};

use crate::ids::{BioProjectAccession, BioSampleAccession, SraRunAccession, SraStudyAccession};
use crate::matrix::ExpressionMatrix;
use crate::measurement::ExpressionMeasurement;
use crate::sample::{BioProject, Sample};
use crate::unit::ExpressionUnit;

/// Query parameters for looking up expression measurements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionQuery {
    pub runs: Option<Vec<SraRunAccession>>,
    pub study: Option<SraStudyAccession>,
    pub bioproject: Option<BioProjectAccession>,
    pub unit: Option<ExpressionUnit>,
    pub limit: Option<usize>,
}

/// Abstract storage layer for expression data.
///
/// Concrete implementations (e.g. a Parquet/DuckDB-backed `expression-store`
/// crate) live outside this crate so the domain types stay I/O-free.
pub trait ExpressionRepository: Send + Sync + 'static {
    fn sample(&self, run: &SraRunAccession) -> Option<Sample>;
    fn samples_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sample>;
    fn samples_for_bioproject(&self, accession: &BioProjectAccession) -> Vec<Sample>;
    fn samples_for_biosample(&self, accession: &BioSampleAccession) -> Vec<Sample>;
    fn bioproject(&self, accession: &BioProjectAccession) -> Option<BioProject>;

    /// Expression values for a single gene, optionally restricted by the
    /// query parameters.
    fn gene_expression(
        &self,
        gene_id: &GeneId,
        query: &ExpressionQuery,
    ) -> Vec<ExpressionMeasurement>;

    /// Dense expression matrix for the given genes × runs in the requested
    /// unit. Returns `None` if the repository cannot satisfy the request
    /// (e.g. some genes or runs are unknown, or the unit is unsupported).
    fn expression_matrix(
        &self,
        accession: &AssemblyAccession,
        gene_ids: &[GeneId],
        runs: &[SraRunAccession],
        unit: ExpressionUnit,
    ) -> Option<ExpressionMatrix>;
}
