mod homology;
mod job;
mod refget;
mod sequence;

use genome_core::{
    Assembly, AssemblyAccession, ClosedRegion, FunctionalAnnotation, Gene, GeneId, GeneRecord,
    GeneSearch, GenomeRepository, KeggEntryId, KeggKoLinks, KeggModule, KeggModuleId, KeggPathway,
    KeggPathwayId, KeggReaction, KeggReactionId, Sequence, TaxId, Taxon, ko_entry_id,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use storage::FastaReference;
use utoipa::ToSchema;

pub use homology::{
    AnnotatedHomologyHit, AnnotatedHomologySearchResult, HomologyAnnotationRepository,
    HomologyService,
};
pub use job::{
    InMemoryJobManager, JobExecutor, JobManager, JobManagerError, JobRecord, JobStatus, Worker,
    WorkerExecutor, WorkerJob,
};

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
    #[error("KEGG pathway not found: {0}")]
    KeggPathwayNotFound(String),
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

    /// Pathway view: pathway info + the KOs that belong to it + the genes in
    /// the dataset annotated with each of those KOs.
    pub fn kegg_pathway(&self, pathway_id: &str) -> Result<KeggPathwayDetail, ServiceError> {
        let pathway_id = KeggPathwayId::new(pathway_id)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        let catalog = self.repository.kegg_catalog();
        let pathway = catalog
            .pathways
            .iter()
            .find(|pathway| pathway.id == pathway_id)
            .cloned()
            .ok_or_else(|| ServiceError::KeggPathwayNotFound(pathway_id.clone().into_string()))?;

        let kos: Vec<KeggEntryId> = catalog
            .ko_links
            .iter()
            .filter(|links| links.pathways.contains(&pathway_id))
            .map(|links| links.ko.clone())
            .collect();
        let ko_views = kos
            .into_iter()
            .map(|ko| KeggPathwayKoEntry {
                genes: self
                    .repository
                    .genes_with_kegg_ko(&ko)
                    .into_iter()
                    .map(KeggGeneSummary::from)
                    .collect(),
                ko,
            })
            .collect();

        Ok(KeggPathwayDetail {
            pathway,
            kos: ko_views,
        })
    }

    /// Per-gene KEGG view that hydrates each KEGG orthology in the gene's
    /// annotations with the pathways/modules/reactions it links to.
    pub fn gene_kegg_view(&self, gene_id: &str) -> Result<GeneKeggView, ServiceError> {
        let gene = self.gene(gene_id)?;
        let catalog = self.repository.kegg_catalog();

        let pathway_names: HashMap<&KeggPathwayId, &Option<String>> = catalog
            .pathways
            .iter()
            .map(|pathway| (&pathway.id, &pathway.name))
            .collect();
        let module_names: HashMap<&KeggModuleId, &Option<String>> = catalog
            .modules
            .iter()
            .map(|module| (&module.id, &module.name))
            .collect();
        let reaction_names: HashMap<&KeggReactionId, &Option<String>> = catalog
            .reactions
            .iter()
            .map(|reaction| (&reaction.id, &reaction.name))
            .collect();
        let links_by_ko: HashMap<&KeggEntryId, &KeggKoLinks> = catalog
            .ko_links
            .iter()
            .map(|links| (&links.ko, links))
            .collect();

        let mut entries = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for annotation in &gene.gene.annotations {
            let FunctionalAnnotation::Kegg(kegg) = annotation else {
                continue;
            };
            let Some(ko) = ko_entry_id(&kegg.entry_id) else {
                continue;
            };
            if !seen.insert(ko.clone()) {
                continue;
            }
            let (pathways, modules, reactions) = links_by_ko
                .get(&ko)
                .map(|links| {
                    let pathways = links
                        .pathways
                        .iter()
                        .map(|id| KeggPathway {
                            id: id.clone(),
                            name: pathway_names.get(id).and_then(|name| (*name).clone()),
                        })
                        .collect();
                    let modules = links
                        .modules
                        .iter()
                        .map(|id| KeggModule {
                            id: id.clone(),
                            name: module_names.get(id).and_then(|name| (*name).clone()),
                        })
                        .collect();
                    let reactions = links
                        .reactions
                        .iter()
                        .map(|id| KeggReaction {
                            id: id.clone(),
                            name: reaction_names.get(id).and_then(|name| (*name).clone()),
                        })
                        .collect();
                    (pathways, modules, reactions)
                })
                .unwrap_or_else(|| (Vec::new(), Vec::new(), Vec::new()));
            entries.push(GeneKeggOrthologyEntry {
                ko,
                name: kegg.name.clone(),
                pathways,
                modules,
                reactions,
            });
        }

        Ok(GeneKeggView {
            gene_id: gene.gene.id,
            entries,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeggPathwayDetail {
    pub pathway: KeggPathway,
    pub kos: Vec<KeggPathwayKoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeggPathwayKoEntry {
    pub ko: KeggEntryId,
    pub genes: Vec<KeggGeneSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeggGeneSummary {
    pub id: GeneId,
    pub symbol: Option<String>,
    pub locus_tag: Option<String>,
}

impl From<Gene> for KeggGeneSummary {
    fn from(gene: Gene) -> Self {
        Self {
            id: gene.id,
            symbol: gene.symbol,
            locus_tag: gene.locus_tag,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeneKeggView {
    pub gene_id: GeneId,
    pub entries: Vec<GeneKeggOrthologyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GeneKeggOrthologyEntry {
    pub ko: KeggEntryId,
    pub name: Option<String>,
    pub pathways: Vec<KeggPathway>,
    pub modules: Vec<KeggModule>,
    pub reactions: Vec<KeggReaction>,
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
            kegg_catalog: genome_core::KeggCatalog::default(),
        };

        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }
}
