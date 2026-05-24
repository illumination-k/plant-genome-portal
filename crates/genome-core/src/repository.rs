use crate::assembly::{Assembly, Sequence, Taxon};
use crate::coord::HalfOpenRegion;
use crate::feature::{Gene, GeneRecord};
use crate::ids::{AssemblyAccession, GeneId, TaxId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneSearch {
    pub tax_id: Option<TaxId>,
    pub symbol: Option<String>,
    pub locus_tag: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

pub trait GenomeRepository: Send + Sync + 'static {
    fn taxon(&self, tax_id: TaxId) -> Option<Taxon>;
    fn assembly(&self, accession: &AssemblyAccession) -> Option<Assembly>;
    fn assemblies_for_taxon(&self, tax_id: TaxId) -> Vec<Assembly>;
    fn sequences_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sequence>;
    fn sequence_by_checksum(&self, checksum: &str) -> Option<Sequence>;
    fn gene(&self, gene_id: &GeneId) -> Option<GeneRecord>;
    fn search_genes(&self, search: &GeneSearch) -> Vec<Gene>;
    fn features_in_region(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<Gene>;
}
