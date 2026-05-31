use expression_domain::{BioProject, ExpressionMatrix, Sample};
use genome_domain::AssemblyAccession;
use serde::{Deserialize, Serialize};

/// Aggregate root for the in-memory expression store, scoped to a single
/// assembly.
///
/// A dataset bundles:
/// - the [`BioProject`]s the samples were collected under,
/// - the per-run [`Sample`] metadata (one entry per SRA Run),
/// - the dense [`ExpressionMatrix`] per supported [`ExpressionUnit`].
///
/// Multi-assembly setups should hold one dataset per assembly and route
/// queries by assembly accession.
///
/// [`ExpressionUnit`]: expression_domain::ExpressionUnit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionDataset {
    pub assembly_accession: AssemblyAccession,
    #[serde(default)]
    pub bioprojects: Vec<BioProject>,
    #[serde(default)]
    pub samples: Vec<Sample>,
    #[serde(default)]
    pub matrices: Vec<ExpressionMatrix>,
}
