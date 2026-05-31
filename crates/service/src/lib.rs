mod homology;
mod job;
mod refget;
mod sequence;

use genome_core::{
    Assembly, AssemblyAccession, ClosedRegion, FunctionalAnnotation, Gene, GeneId, GeneRecord,
    GeneSearch, GenomeRepository, HalfOpenRegion, KeggEntryId, KeggKoLinks, KeggModule,
    KeggModuleId, KeggPathway, KeggPathwayId, KeggReaction, KeggReactionId, Orthogroup,
    OrthogroupId, Position0, Sequence, Strand, TaxId, Taxon, Transcript, TranscriptId, ko_entry_id,
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
pub use sequence::TranscriptProtein;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("taxon not found: {0}")]
    TaxonNotFound(TaxId),
    #[error("assembly not found: {0}")]
    AssemblyNotFound(String),
    #[error("gene not found: {0}")]
    GeneNotFound(String),
    #[error("transcript not found: {0}")]
    TranscriptNotFound(String),
    #[error("sequence not found: {0}")]
    SequenceNotFound(String),
    #[error("protein sequence not available: {0}")]
    ProteinSequenceUnavailable(String),
    #[error("KEGG pathway not found: {0}")]
    KeggPathwayNotFound(String),
    #[error("orthogroup not found: {0}")]
    OrthogroupNotFound(String),
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

    pub fn transcript(&self, transcript_id: &str) -> Result<Transcript, ServiceError> {
        let transcript_id = TranscriptId::new(transcript_id)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        self.repository
            .transcript(&transcript_id)
            .ok_or_else(|| ServiceError::TranscriptNotFound(transcript_id.into_string()))
    }

    pub fn search_genes(&self, search: GeneSearch) -> Vec<Gene> {
        self.repository.search_genes(&search)
    }

    /// KEGG pathway catalog summary, including how many KOs and dataset genes
    /// currently connect to each pathway.
    pub fn kegg_pathways(&self) -> Vec<KeggPathwaySummary> {
        let catalog = self.repository.kegg_catalog();
        let mut summaries = catalog
            .pathways
            .iter()
            .map(|pathway| {
                let ko_links = catalog
                    .ko_links
                    .iter()
                    .filter(|links| links.pathways.contains(&pathway.id))
                    .collect::<Vec<_>>();
                let mut genes = std::collections::HashSet::new();
                for links in &ko_links {
                    for gene in self.repository.genes_with_kegg_ko(&links.ko) {
                        genes.insert(gene.id);
                    }
                }
                KeggPathwaySummary {
                    pathway: pathway.clone(),
                    ko_count: ko_links.len(),
                    gene_count: genes.len(),
                }
            })
            .collect::<Vec<_>>();
        summaries.sort_by(|left, right| left.pathway.id.cmp(&right.pathway.id));
        summaries
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

    /// Returns the gene body extended by `upstream_bp` on the 5' side and
    /// `downstream_bp` on the 3' side (relative to the gene's strand), clamped
    /// to the sequence boundaries.
    ///
    /// Used by the gene-centric epigenome view to find peaks overlapping the
    /// gene + its promoter / terminator flanks.
    pub fn gene_flank_region(
        &self,
        gene_id: &str,
        upstream_bp: u64,
        downstream_bp: u64,
    ) -> Result<(GeneRecord, HalfOpenRegion), ServiceError> {
        let record = self.gene(gene_id)?;
        let gene = &record.gene;
        let sequence_length = self
            .repository
            .sequences_for_assembly(&gene.assembly_accession)
            .into_iter()
            .find(|sequence| sequence.name == gene.sequence_name)
            .map(|sequence| sequence.length)
            .ok_or_else(|| {
                ServiceError::SequenceNotFound(gene.sequence_name.as_str().to_owned())
            })?;

        let (left_flank, right_flank) = match gene.strand {
            Strand::Reverse => (downstream_bp, upstream_bp),
            // Treat Unknown like Forward: callers asked for upstream/downstream
            // in genomic coordinates when strand is unknown.
            Strand::Forward | Strand::Unknown => (upstream_bp, downstream_bp),
        };

        let start = gene.region.start.get().saturating_sub(left_flank);
        let end = gene
            .region
            .end
            .get()
            .saturating_add(right_flank)
            .min(sequence_length);

        let region = HalfOpenRegion::new(
            gene.sequence_name.clone(),
            Position0::new(start),
            Position0::new(end),
        )
        .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;

        Ok((record, region))
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

    pub fn gene_orthogroups(&self, gene_id: &str) -> Result<Vec<Orthogroup>, ServiceError> {
        let gene_id = GeneId::new(gene_id)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        if self.repository.gene(&gene_id).is_none() {
            return Err(ServiceError::GeneNotFound(gene_id.into_string()));
        }
        Ok(self.repository.orthogroups_for_gene(&gene_id))
    }

    pub fn orthogroup(&self, orthogroup_id: &str) -> Result<Orthogroup, ServiceError> {
        let orthogroup_id = OrthogroupId::new(orthogroup_id)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        self.repository
            .orthogroup(&orthogroup_id)
            .ok_or_else(|| ServiceError::OrthogroupNotFound(orthogroup_id.into_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeggPathwayDetail {
    pub pathway: KeggPathway,
    pub kos: Vec<KeggPathwayKoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KeggPathwaySummary {
    pub pathway: KeggPathway,
    pub ko_count: usize,
    pub gene_count: usize,
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

    #[test]
    fn gene_orthogroups_returns_empty_for_gene_without_membership() {
        let service = make_service();

        let groups = service.gene_orthogroups("Mp1g00010").unwrap();

        assert!(groups.is_empty());
    }

    #[test]
    fn gene_orthogroups_returns_404_for_missing_gene() {
        let service = make_service();

        let error = service.gene_orthogroups("Mp9g99999").unwrap_err();

        assert!(matches!(error, ServiceError::GeneNotFound(_)));
    }

    #[test]
    fn orthogroup_returns_group_by_id() {
        let service = orthogroup_service();

        let group = service.orthogroup("OG1").unwrap();

        assert_eq!(group.id.as_str(), "OG1");
        assert_eq!(group.members.len(), 1);
        assert!(matches!(
            service.orthogroup("OG404").unwrap_err(),
            ServiceError::OrthogroupNotFound(_)
        ));
    }

    #[test]
    fn gene_orthogroups_returns_membership_for_existing_gene() {
        let service = orthogroup_service();

        let groups = service.gene_orthogroups("Mp1g00010").unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].id.as_str(), "OG1");
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
            orthogroup_catalog: genome_core::OrthogroupCatalog::default(),
        };

        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }

    fn orthogroup_service() -> GenomeService<FileGenomeRepository> {
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
                assembly_accession: accession.clone(),
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
            orthogroup_catalog: genome_core::OrthogroupCatalog {
                groups: vec![genome_core::Orthogroup {
                    id: genome_core::OrthogroupId::new("OG1").unwrap(),
                    members: vec![genome_core::OrthogroupMember {
                        gene_id: GeneId::new("Mp1g00010").unwrap(),
                        tax_id: TaxId::new(3197),
                        scientific_name: "Marchantia polymorpha".to_owned(),
                        assembly_accession: Some(accession),
                        symbol: Some("FOO".to_owned()),
                    }],
                }],
            },
        };
        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }

    fn kegg_gene(id: &str, kos: &[&str]) -> Gene {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        Gene {
            id: GeneId::new(id).unwrap(),
            assembly_accession: accession,
            symbol: Some(format!("SYMB_{id}")),
            locus_tag: Some(format!("LOC_{id}")),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: HalfOpenRegion::new(
                SequenceName::new("chr1").unwrap(),
                Position0::new(0),
                Position0::new(10),
            )
            .unwrap(),
            strand: Strand::Forward,
            feature_type: "gene".to_owned(),
            annotations: kos
                .iter()
                .map(|ko| {
                    FunctionalAnnotation::Kegg(genome_core::KeggAnnotation::new(
                        KeggEntryId::new(*ko).unwrap(),
                        Some(format!("name of {ko}")),
                        genome_core::AnnotationEvidence::new(genome_core::AnnotationSource::Kegg),
                    ))
                })
                .collect(),
            attributes: BTreeMap::new(),
        }
    }

    fn kegg_service() -> GenomeService<FileGenomeRepository> {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let catalog = genome_core::KeggCatalog {
            pathways: vec![
                KeggPathway {
                    id: KeggPathwayId::new("map00010").unwrap(),
                    name: Some("Glycolysis".to_owned()),
                },
                KeggPathway {
                    id: KeggPathwayId::new("map00020").unwrap(),
                    name: Some("TCA cycle".to_owned()),
                },
            ],
            modules: vec![KeggModule {
                id: KeggModuleId::new("M00001").unwrap(),
                name: Some("Module-one".to_owned()),
            }],
            reactions: vec![KeggReaction {
                id: KeggReactionId::new("R00754").unwrap(),
                name: Some("Reaction-one".to_owned()),
            }],
            ko_links: vec![
                genome_core::KeggKoLinks {
                    ko: KeggEntryId::new("K00001").unwrap(),
                    pathways: vec![KeggPathwayId::new("map00010").unwrap()],
                    modules: vec![KeggModuleId::new("M00001").unwrap()],
                    reactions: vec![KeggReactionId::new("R00754").unwrap()],
                },
                genome_core::KeggKoLinks {
                    ko: KeggEntryId::new("K00002").unwrap(),
                    pathways: vec![KeggPathwayId::new("map00010").unwrap()],
                    modules: Vec::new(),
                    reactions: Vec::new(),
                },
                genome_core::KeggKoLinks {
                    ko: KeggEntryId::new("K00003").unwrap(),
                    pathways: vec![KeggPathwayId::new("map00020").unwrap()],
                    modules: Vec::new(),
                    reactions: Vec::new(),
                },
            ],
        };
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
            // gene_a covers K00001 (and a duplicate `ko:K00001`).
            // gene_b covers K00002.
            // gene_c covers K00003 (only on map00020).
            genes: vec![
                kegg_gene("MpAAA", &["K00001", "ko:K00001"]),
                kegg_gene("MpBBB", &["K00002"]),
                kegg_gene("MpCCC", &["K00003"]),
            ],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: catalog,
            orthogroup_catalog: genome_core::OrthogroupCatalog::default(),
        };
        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }

    #[test]
    fn kegg_pathway_returns_kos_in_pathway_with_matching_genes() {
        let service = kegg_service();
        let detail = service.kegg_pathway("map00010").unwrap();

        assert_eq!(detail.pathway.id.as_str(), "map00010");
        assert_eq!(detail.pathway.name.as_deref(), Some("Glycolysis"));
        // Both K00001 and K00002 link to map00010; K00003 does not.
        let kos: Vec<&str> = detail.kos.iter().map(|k| k.ko.as_str()).collect();
        assert!(kos.contains(&"K00001"));
        assert!(kos.contains(&"K00002"));
        assert!(!kos.contains(&"K00003"));

        // The K00001 entry must include MpAAA (which has K00001) and not MpBBB/MpCCC.
        let k1 = detail
            .kos
            .iter()
            .find(|k| k.ko.as_str() == "K00001")
            .unwrap();
        assert_eq!(k1.genes.len(), 1);
        assert_eq!(k1.genes[0].id.as_str(), "MpAAA");

        // The K00002 entry must include MpBBB only.
        let k2 = detail
            .kos
            .iter()
            .find(|k| k.ko.as_str() == "K00002")
            .unwrap();
        assert_eq!(k2.genes.len(), 1);
        assert_eq!(k2.genes[0].id.as_str(), "MpBBB");
    }

    #[test]
    fn kegg_pathways_returns_catalog_with_ko_and_gene_counts() {
        let service = kegg_service();
        let pathways = service.kegg_pathways();

        assert_eq!(pathways.len(), 2);
        assert_eq!(pathways[0].pathway.id.as_str(), "map00010");
        assert_eq!(pathways[0].pathway.name.as_deref(), Some("Glycolysis"));
        assert_eq!(pathways[0].ko_count, 2);
        assert_eq!(pathways[0].gene_count, 2);
        assert_eq!(pathways[1].pathway.id.as_str(), "map00020");
        assert_eq!(pathways[1].ko_count, 1);
        assert_eq!(pathways[1].gene_count, 1);
    }

    #[test]
    fn kegg_pathway_canonicalizes_input_id() {
        // `ko00010` should canonicalize to `map00010`.
        let service = kegg_service();
        let detail = service.kegg_pathway("ko00010").unwrap();
        assert_eq!(detail.pathway.id.as_str(), "map00010");
    }

    #[test]
    fn kegg_pathway_returns_not_found_for_missing_id() {
        let service = kegg_service();
        let err = service.kegg_pathway("map99999").unwrap_err();
        assert!(matches!(err, ServiceError::KeggPathwayNotFound(_)));
    }

    #[test]
    fn kegg_pathway_returns_invalid_request_for_bad_id() {
        let service = kegg_service();
        let err = service.kegg_pathway("not-a-pathway").unwrap_err();
        assert!(matches!(err, ServiceError::InvalidRequest(_)));
    }

    #[test]
    fn gene_kegg_view_dedups_ko_and_hydrates_names() {
        let service = kegg_service();
        let view = service.gene_kegg_view("MpAAA").unwrap();

        // MpAAA has both `K00001` and `ko:K00001` — they collapse to one entry.
        assert_eq!(view.gene_id.as_str(), "MpAAA");
        assert_eq!(view.entries.len(), 1);
        let entry = &view.entries[0];
        assert_eq!(entry.ko.as_str(), "K00001");

        // Pathway/module/reaction names come from the catalog.
        assert_eq!(entry.pathways.len(), 1);
        assert_eq!(entry.pathways[0].id.as_str(), "map00010");
        assert_eq!(entry.pathways[0].name.as_deref(), Some("Glycolysis"));
        assert_eq!(entry.modules.len(), 1);
        assert_eq!(entry.modules[0].name.as_deref(), Some("Module-one"));
        assert_eq!(entry.reactions.len(), 1);
        assert_eq!(entry.reactions[0].name.as_deref(), Some("Reaction-one"));
    }

    #[test]
    fn gene_kegg_view_returns_empty_links_for_ko_with_no_catalog_entry() {
        let service = kegg_service();
        // MpBBB only has K00002 which links to map00010 but no module/reaction.
        let view = service.gene_kegg_view("MpBBB").unwrap();
        assert_eq!(view.entries.len(), 1);
        assert_eq!(view.entries[0].ko.as_str(), "K00002");
        assert_eq!(view.entries[0].pathways.len(), 1);
        assert_eq!(view.entries[0].pathways[0].id.as_str(), "map00010");
        assert!(view.entries[0].modules.is_empty());
        assert!(view.entries[0].reactions.is_empty());
    }

    #[test]
    fn gene_kegg_view_returns_not_found_for_missing_gene() {
        let service = kegg_service();
        let err = service.gene_kegg_view("MpZZZ").unwrap_err();
        assert!(matches!(err, ServiceError::GeneNotFound(_)));
    }

    fn flank_service(strand: Strand) -> GenomeService<FileGenomeRepository> {
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
            sequences: vec![Sequence {
                name: SequenceName::new("chr1").unwrap(),
                assembly_accession: accession.clone(),
                length: 10_000,
                refget_checksum: String::new(),
            }],
            genes: vec![Gene {
                id: GeneId::new("MpFlank").unwrap(),
                assembly_accession: accession,
                symbol: None,
                locus_tag: None,
                sequence_name: SequenceName::new("chr1").unwrap(),
                region: HalfOpenRegion::new(
                    SequenceName::new("chr1").unwrap(),
                    Position0::new(3_000),
                    Position0::new(4_000),
                )
                .unwrap(),
                strand,
                feature_type: "gene".to_owned(),
                annotations: Vec::new(),
                attributes: BTreeMap::new(),
            }],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_core::KeggCatalog::default(),
            orthogroup_catalog: genome_core::OrthogroupCatalog::default(),
        };

        GenomeService::new(FileGenomeRepository::new(dataset), None)
    }

    #[test]
    fn gene_flank_region_extends_forward_strand_upstream() {
        let service = flank_service(Strand::Forward);
        let (_, region) = service.gene_flank_region("MpFlank", 2_000, 100).unwrap();
        // Forward: [start - upstream, end + downstream) = [1000, 4100)
        assert_eq!(region.start.get(), 1_000);
        assert_eq!(region.end.get(), 4_100);
    }

    #[test]
    fn gene_flank_region_swaps_flanks_for_reverse_strand() {
        let service = flank_service(Strand::Reverse);
        let (_, region) = service.gene_flank_region("MpFlank", 2_000, 100).unwrap();
        // Reverse: [start - downstream, end + upstream) = [2900, 6000)
        assert_eq!(region.start.get(), 2_900);
        assert_eq!(region.end.get(), 6_000);
    }

    #[test]
    fn gene_flank_region_clamps_to_sequence_boundaries() {
        let service = flank_service(Strand::Forward);
        // upstream larger than the gene's start → clamp to 0; downstream larger
        // than the remaining sequence → clamp to sequence length.
        let (_, region) = service
            .gene_flank_region("MpFlank", 10_000, 10_000_000)
            .unwrap();
        assert_eq!(region.start.get(), 0);
        assert_eq!(region.end.get(), 10_000);
    }

    #[test]
    fn gene_flank_region_treats_unknown_strand_as_forward() {
        let service = flank_service(Strand::Unknown);
        let (_, region) = service.gene_flank_region("MpFlank", 2_000, 100).unwrap();
        assert_eq!(region.start.get(), 1_000);
        assert_eq!(region.end.get(), 4_100);
    }

    #[test]
    fn gene_flank_region_returns_not_found_for_missing_gene() {
        let service = flank_service(Strand::Forward);
        let err = service
            .gene_flank_region("MpGhost", 1_000, 1_000)
            .unwrap_err();
        assert!(matches!(err, ServiceError::GeneNotFound(_)));
    }
}
