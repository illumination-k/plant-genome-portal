use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ids::{AssemblyAccession, GeneId, OrthogroupId, TaxId};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub struct OrthogroupCatalog {
    pub groups: Vec<Orthogroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Orthogroup {
    pub id: OrthogroupId,
    pub members: Vec<OrthogroupMember>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct OrthogroupMember {
    pub gene_id: GeneId,
    pub tax_id: TaxId,
    pub scientific_name: String,
    pub assembly_accession: Option<AssemblyAccession>,
    pub symbol: Option<String>,
}
