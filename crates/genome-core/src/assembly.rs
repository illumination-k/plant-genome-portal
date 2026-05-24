use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ids::{AssemblyAccession, SequenceName, TaxId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssemblySource {
    Ncbi,
    MarpolBase,
    Tair,
    Phytozome,
    Community,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Taxon {
    pub tax_id: TaxId,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub rank: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Assembly {
    pub accession: AssemblyAccession,
    pub tax_id: TaxId,
    pub name: String,
    pub source: AssemblySource,
    pub refget_checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Sequence {
    pub name: SequenceName,
    pub assembly_accession: AssemblyAccession,
    pub length: u64,
    pub refget_checksum: String,
}
