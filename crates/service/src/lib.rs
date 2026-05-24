mod homology;
mod job;

use genome_core::{
    Assembly, AssemblyAccession, ClosedRegion, Gene, GeneId, GeneRecord, GeneSearch,
    GenomeRepository, Sequence, TaxId, Taxon,
};
use std::str::FromStr;
use std::sync::Arc;
use storage::FastaReference;

pub use homology::{
    AnnotatedHomologyHit, AnnotatedHomologySearchResult, HomologyAnnotationRepository,
    HomologyService,
};
pub use job::{Worker, WorkerJob};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("taxon not found: {0}")]
    TaxonNotFound(TaxId),
    #[error("assembly not found: {0}")]
    AssemblyNotFound(String),
    #[error("gene not found: {0}")]
    GeneNotFound(String),
    #[error("sequence not found: {0}")]
    SequenceNotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

#[derive(Clone)]
pub struct GenomeService<R> {
    repository: Arc<R>,
    reference: Option<Arc<FastaReference>>,
}

impl<R> GenomeService<R>
where
    R: GenomeRepository,
{
    pub fn new(repository: R, reference: Option<FastaReference>) -> Self {
        Self {
            repository: Arc::new(repository),
            reference: reference.map(Arc::new),
        }
    }

    pub fn taxon(&self, tax_id: TaxId) -> Result<Taxon, ServiceError> {
        self.repository
            .taxon(tax_id)
            .ok_or(ServiceError::TaxonNotFound(tax_id))
    }

    pub fn assembly(&self, accession: &str) -> Result<Assembly, ServiceError> {
        let accession = AssemblyAccession::new(accession)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        self.repository
            .assembly(&accession)
            .ok_or_else(|| ServiceError::AssemblyNotFound(accession.into_string()))
    }

    pub fn assemblies_for_taxon(&self, tax_id: TaxId) -> Vec<Assembly> {
        self.repository.assemblies_for_taxon(tax_id)
    }

    pub fn sequences_for_assembly(&self, accession: &str) -> Result<Vec<Sequence>, ServiceError> {
        let accession = AssemblyAccession::new(accession)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        if self.repository.assembly(&accession).is_none() {
            return Err(ServiceError::AssemblyNotFound(accession.into_string()));
        }
        Ok(self.repository.sequences_for_assembly(&accession))
    }

    pub fn gene(&self, gene_id: &str) -> Result<GeneRecord, ServiceError> {
        let gene_id = GeneId::new(gene_id)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        self.repository
            .gene(&gene_id)
            .ok_or_else(|| ServiceError::GeneNotFound(gene_id.into_string()))
    }

    pub fn search_genes(&self, search: GeneSearch) -> Vec<Gene> {
        self.repository.search_genes(&search)
    }

    pub fn features_in_region(
        &self,
        accession: &str,
        region: &str,
    ) -> Result<Vec<Gene>, ServiceError> {
        let accession = AssemblyAccession::new(accession)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        let region = ClosedRegion::from_str(region)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?
            .to_half_open()
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;

        if self.repository.assembly(&accession).is_none() {
            return Err(ServiceError::AssemblyNotFound(accession.into_string()));
        }

        Ok(self.repository.features_in_region(&accession, &region))
    }

    pub fn refget_sequence(
        &self,
        checksum: &str,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<String, ServiceError> {
        if self.repository.sequence_by_checksum(checksum).is_none() {
            return Err(ServiceError::SequenceNotFound(checksum.to_owned()));
        }

        let reference = self
            .reference
            .as_ref()
            .ok_or_else(|| ServiceError::SequenceNotFound(checksum.to_owned()))?;

        reference
            .get(checksum, start, end)
            .ok_or_else(|| ServiceError::InvalidRequest("invalid sequence range".to_owned()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use genome_core::{
        AssemblySource, GenomeDataset, HalfOpenRegion, Position0, SequenceName, Strand,
    };
    use std::collections::BTreeMap;
    use storage::FileGenomeRepository;

    #[test]
    fn search_uses_repository_trait() {
        let service = make_service();

        let results = service.search_genes(GeneSearch {
            query: Some("foo".to_owned()),
            ..GeneSearch::default()
        });

        assert_eq!(results.len(), 1);
    }

    fn make_service() -> GenomeService<FileGenomeRepository> {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let dataset = GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(3197),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: accession.clone(),
                tax_id: TaxId::new(3197),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: Vec::new(),
            genes: vec![Gene {
                id: GeneId::new("Mp1g00010").unwrap(),
                assembly_accession: accession,
                symbol: Some("FOO".to_owned()),
                locus_tag: None,
                sequence_name: SequenceName::new("chr1").unwrap(),
                region: HalfOpenRegion::new(
                    SequenceName::new("chr1").unwrap(),
                    Position0::new(0),
                    Position0::new(10),
                )
                .unwrap(),
                strand: Strand::Forward,
                feature_type: "gene".to_owned(),
                annotations: Vec::new(),
                attributes: BTreeMap::new(),
            }],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
        };

        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }
}
