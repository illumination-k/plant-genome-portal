use std::collections::HashMap;
use std::path::Path;

use genome_domain::{
    Assembly, AssemblyAccession, Cds, Exon, FunctionalAnnotation, Gene, GeneId, GeneRecord,
    GeneSearch, GenomeDataset, GenomeRepository, HalfOpenRegion, KeggCatalog, KeggEntryId,
    Orthogroup, OrthogroupId, Sequence, TaxId, Taxon, Transcript, TranscriptId, ko_entry_id,
};

use crate::error::StorageError;
use crate::snapshot::read_snapshot;

#[derive(Debug, Clone)]
pub struct FileGenomeRepository {
    dataset: GenomeDataset,
    genes: GeneIndex,
    transcripts: TranscriptIndex,
    sequences: SequenceIndex,
    kegg: KeggKoIndex,
    orthogroups: OrthogroupIndex,
}

#[derive(Debug, Clone)]
struct GeneIndex {
    by_id: HashMap<GeneId, Gene>,
}

impl GeneIndex {
    fn new(genes: &[Gene]) -> Self {
        Self {
            by_id: genes
                .iter()
                .map(|gene| (gene.id.clone(), gene.clone()))
                .collect(),
        }
    }

    fn get(&self, gene_id: &GeneId) -> Option<Gene> {
        self.by_id.get(gene_id).cloned()
    }
}

#[derive(Debug, Clone)]
struct TranscriptIndex {
    by_id: HashMap<TranscriptId, Transcript>,
    by_gene: HashMap<GeneId, Vec<Transcript>>,
    exons_by_transcript: HashMap<TranscriptId, Vec<Exon>>,
    cdss_by_transcript: HashMap<TranscriptId, Vec<Cds>>,
}

impl TranscriptIndex {
    fn new(transcripts: &[Transcript], exons: &[Exon], cdss: &[Cds]) -> Self {
        let mut by_gene: HashMap<GeneId, Vec<Transcript>> = HashMap::new();
        let mut by_id: HashMap<TranscriptId, Transcript> = HashMap::new();
        for transcript in transcripts {
            by_gene
                .entry(transcript.gene_id.clone())
                .or_default()
                .push(transcript.clone());
            by_id.insert(transcript.id.clone(), transcript.clone());
        }

        let mut exons_by_transcript: HashMap<TranscriptId, Vec<Exon>> = HashMap::new();
        for exon in exons {
            exons_by_transcript
                .entry(exon.transcript_id.clone())
                .or_default()
                .push(exon.clone());
        }

        let mut cdss_by_transcript: HashMap<TranscriptId, Vec<Cds>> = HashMap::new();
        for cds in cdss {
            cdss_by_transcript
                .entry(cds.transcript_id.clone())
                .or_default()
                .push(cds.clone());
        }

        Self {
            by_id,
            by_gene,
            exons_by_transcript,
            cdss_by_transcript,
        }
    }

    fn get(&self, transcript_id: &TranscriptId) -> Option<Transcript> {
        self.by_id.get(transcript_id).cloned()
    }

    fn record_parts_for_gene(&self, gene_id: &GeneId) -> (Vec<Transcript>, Vec<Exon>, Vec<Cds>) {
        let transcripts = self.by_gene.get(gene_id).cloned().unwrap_or_default();
        let exons = transcripts
            .iter()
            .flat_map(|transcript| {
                self.exons_by_transcript
                    .get(&transcript.id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        let cdss = transcripts
            .iter()
            .flat_map(|transcript| {
                self.cdss_by_transcript
                    .get(&transcript.id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();

        (transcripts, exons, cdss)
    }
}

#[derive(Debug, Clone)]
struct SequenceIndex {
    by_checksum: HashMap<String, Sequence>,
}

impl SequenceIndex {
    fn new(sequences: &[Sequence]) -> Self {
        Self {
            by_checksum: sequences
                .iter()
                .map(|sequence| (sequence.refget_checksum.clone(), sequence.clone()))
                .collect(),
        }
    }

    fn by_checksum(&self, checksum: &str) -> Option<Sequence> {
        self.by_checksum.get(checksum).cloned()
    }
}

#[derive(Debug, Clone)]
struct KeggKoIndex {
    genes_by_ko: HashMap<KeggEntryId, Vec<GeneId>>,
}

impl KeggKoIndex {
    fn new(genes: &[Gene]) -> Self {
        Self {
            genes_by_ko: build_kegg_ko_index(genes),
        }
    }

    fn genes_with_ko(&self, ko: &KeggEntryId, genes: &GeneIndex) -> Vec<Gene> {
        self.genes_by_ko
            .get(ko)
            .map(|gene_ids| {
                gene_ids
                    .iter()
                    .filter_map(|gene_id| genes.get(gene_id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
struct OrthogroupIndex {
    by_id: HashMap<OrthogroupId, Orthogroup>,
    by_gene: HashMap<GeneId, Vec<OrthogroupId>>,
}

impl OrthogroupIndex {
    fn new(groups: &[Orthogroup]) -> Self {
        let mut by_id = HashMap::new();
        let mut by_gene: HashMap<GeneId, Vec<OrthogroupId>> = HashMap::new();
        for group in groups {
            by_id.insert(group.id.clone(), group.clone());
            for member in &group.members {
                by_gene
                    .entry(member.gene_id.clone())
                    .or_default()
                    .push(group.id.clone());
            }
        }
        for group_ids in by_gene.values_mut() {
            group_ids.sort();
            group_ids.dedup();
        }
        Self { by_id, by_gene }
    }

    fn groups_for_gene(&self, gene_id: &GeneId) -> Vec<Orthogroup> {
        self.by_gene
            .get(gene_id)
            .map(|group_ids| {
                group_ids
                    .iter()
                    .filter_map(|group_id| self.by_id.get(group_id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get(&self, orthogroup_id: &OrthogroupId) -> Option<Orthogroup> {
        self.by_id.get(orthogroup_id).cloned()
    }
}

impl FileGenomeRepository {
    pub fn new(dataset: GenomeDataset) -> Self {
        let genes = GeneIndex::new(&dataset.genes);
        let transcripts = TranscriptIndex::new(&dataset.transcripts, &dataset.exons, &dataset.cdss);
        let sequences = SequenceIndex::new(&dataset.sequences);
        let kegg = KeggKoIndex::new(&dataset.genes);
        let orthogroups = OrthogroupIndex::new(&dataset.orthogroup_catalog.groups);

        Self {
            dataset,
            genes,
            transcripts,
            sequences,
            kegg,
            orthogroups,
        }
    }

    pub fn from_snapshot_path(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let snapshot = read_snapshot(path)?;
        Ok(Self::new(snapshot.dataset))
    }

    pub fn default_assembly_accession(&self) -> AssemblyAccession {
        self.dataset.assembly.accession.clone()
    }
}

impl GenomeRepository for FileGenomeRepository {
    fn taxon(&self, tax_id: TaxId) -> Option<Taxon> {
        (self.dataset.taxon.tax_id == tax_id).then(|| self.dataset.taxon.clone())
    }

    fn assembly(&self, accession: &AssemblyAccession) -> Option<Assembly> {
        (&self.dataset.assembly.accession == accession).then(|| self.dataset.assembly.clone())
    }

    fn assemblies_for_taxon(&self, tax_id: TaxId) -> Vec<Assembly> {
        if self.dataset.assembly.tax_id == tax_id {
            vec![self.dataset.assembly.clone()]
        } else {
            Vec::new()
        }
    }

    fn sequences_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sequence> {
        if &self.dataset.assembly.accession == accession {
            self.dataset.sequences.clone()
        } else {
            Vec::new()
        }
    }

    fn sequence_by_checksum(&self, checksum: &str) -> Option<Sequence> {
        self.sequences.by_checksum(checksum)
    }

    fn gene(&self, gene_id: &GeneId) -> Option<GeneRecord> {
        let gene = self.genes.get(gene_id)?;
        let (transcripts, exons, cdss) = self.transcripts.record_parts_for_gene(gene_id);

        Some(GeneRecord {
            gene,
            transcripts,
            exons,
            cdss,
        })
    }

    fn transcript(&self, transcript_id: &TranscriptId) -> Option<Transcript> {
        self.transcripts.get(transcript_id)
    }

    fn search_genes(&self, search: &GeneSearch) -> Vec<Gene> {
        let limit = search.limit.unwrap_or(50);
        self.dataset
            .genes
            .iter()
            .filter(|gene| {
                search
                    .tax_id
                    .is_none_or(|tax_id| self.dataset.taxon.tax_id == tax_id)
                    && search_symbol(gene, search.symbol.as_deref())
                    && search_locus_tag(gene, search.locus_tag.as_deref())
                    && search_query(gene, search.query.as_deref())
            })
            .take(limit)
            .cloned()
            .collect()
    }

    fn features_in_region(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<Gene> {
        if &self.dataset.assembly.accession != accession {
            return Vec::new();
        }

        self.dataset
            .genes
            .iter()
            .filter(|gene| gene.region.overlaps(region))
            .cloned()
            .collect()
    }

    fn kegg_catalog(&self) -> &KeggCatalog {
        &self.dataset.kegg_catalog
    }

    fn genes_with_kegg_ko(&self, ko: &KeggEntryId) -> Vec<Gene> {
        self.kegg.genes_with_ko(ko, &self.genes)
    }

    fn orthogroups_for_gene(&self, gene_id: &GeneId) -> Vec<Orthogroup> {
        self.orthogroups.groups_for_gene(gene_id)
    }

    fn orthogroup(&self, orthogroup_id: &OrthogroupId) -> Option<Orthogroup> {
        self.orthogroups.get(orthogroup_id)
    }
}

fn build_kegg_ko_index(genes: &[Gene]) -> HashMap<KeggEntryId, Vec<GeneId>> {
    let mut index: HashMap<KeggEntryId, Vec<GeneId>> = HashMap::new();
    for gene in genes {
        let mut seen = std::collections::HashSet::new();
        for annotation in &gene.annotations {
            if let FunctionalAnnotation::Kegg(kegg) = annotation
                && let Some(ko) = ko_entry_id(&kegg.entry_id)
                && seen.insert(ko.clone())
            {
                index.entry(ko).or_default().push(gene.id.clone());
            }
        }
    }
    index
}

fn search_symbol(gene: &Gene, symbol: Option<&str>) -> bool {
    symbol.is_none_or(|symbol| {
        gene.symbol
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(symbol))
    })
}

fn search_locus_tag(gene: &Gene, locus_tag: Option<&str>) -> bool {
    locus_tag.is_none_or(|locus_tag| {
        gene.locus_tag
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case(locus_tag))
    })
}

fn search_query(gene: &Gene, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };
    let query = query.to_ascii_lowercase();
    gene.id.as_str().to_ascii_lowercase().contains(&query)
        || gene
            .symbol
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || gene
            .locus_tag
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || gene
            .attributes
            .values()
            .any(|value| value.to_ascii_lowercase().contains(&query))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;

    use genome_domain::{
        Assembly, AssemblySource, Cds, Exon, GenomeDataset, HalfOpenRegion, Position0,
        SequenceName, Strand, Taxon, Transcript,
    };

    use super::*;

    const TAX_ID: u32 = 3197;

    fn region(start: u64, end: u64) -> HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(start),
            Position0::new(end),
        )
        .unwrap()
    }

    fn make_repository() -> FileGenomeRepository {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let sequence = Sequence {
            name: SequenceName::new("chr1").unwrap(),
            assembly_accession: accession.clone(),
            length: 1000,
            refget_checksum: "chk-chr1".to_owned(),
        };
        let gene_a = Gene {
            id: GeneId::new("Mp1g00010").unwrap(),
            assembly_accession: accession.clone(),
            symbol: Some("MpFOO".to_owned()),
            locus_tag: Some("LOCUS1".to_owned()),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(0, 100),
            strand: Strand::Forward,
            feature_type: "gene".to_owned(),
            annotations: Vec::new(),
            attributes: {
                let mut attrs = BTreeMap::new();
                attrs.insert("note".to_owned(), "interesting-bar".to_owned());
                attrs
            },
        };
        let gene_b = Gene {
            id: GeneId::new("Mp1g00020").unwrap(),
            assembly_accession: accession.clone(),
            symbol: Some("MpBAZ".to_owned()),
            locus_tag: Some("LOCUS2".to_owned()),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(500, 600),
            strand: Strand::Reverse,
            feature_type: "gene".to_owned(),
            annotations: Vec::new(),
            attributes: BTreeMap::new(),
        };
        let transcript = Transcript {
            id: TranscriptId::new("Mp1g00010.1").unwrap(),
            gene_id: GeneId::new("Mp1g00010").unwrap(),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(0, 100),
            strand: Strand::Forward,
            feature_type: "mRNA".to_owned(),
            annotations: Vec::new(),
            attributes: BTreeMap::new(),
            protein_checksum: None,
            protein_length: None,
        };
        let exon = Exon {
            transcript_id: TranscriptId::new("Mp1g00010.1").unwrap(),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(0, 50),
            strand: Strand::Forward,
        };
        let cds = Cds {
            transcript_id: TranscriptId::new("Mp1g00010.1").unwrap(),
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(10, 40),
            strand: Strand::Forward,
            phase: Some(0),
        };

        FileGenomeRepository::new(GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(TAX_ID),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: accession.clone(),
                tax_id: TaxId::new(TAX_ID),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: vec![sequence],
            genes: vec![gene_a, gene_b],
            transcripts: vec![transcript],
            exons: vec![exon],
            cdss: vec![cds],
            kegg_catalog: KeggCatalog::default(),
            orthogroup_catalog: genome_domain::OrthogroupCatalog::default(),
        })
    }

    #[test]
    fn taxon_returns_match_and_none_for_other_id() {
        let repo = make_repository();
        assert_eq!(repo.taxon(TaxId::new(TAX_ID)).unwrap().tax_id.get(), TAX_ID);
        assert!(repo.taxon(TaxId::new(9999)).is_none());
    }

    #[test]
    fn assembly_returns_match_and_none_for_other_accession() {
        let repo = make_repository();
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert_eq!(repo.assembly(&accession).unwrap().accession, accession);
        assert!(repo.assembly(&other).is_none());
    }

    #[test]
    fn assemblies_for_taxon_filters_by_tax_id() {
        let repo = make_repository();
        assert_eq!(repo.assemblies_for_taxon(TaxId::new(TAX_ID)).len(), 1);
        assert!(repo.assemblies_for_taxon(TaxId::new(9999)).is_empty());
    }

    #[test]
    fn sequences_for_assembly_filters_by_accession() {
        let repo = make_repository();
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert_eq!(repo.sequences_for_assembly(&accession).len(), 1);
        assert!(repo.sequences_for_assembly(&other).is_empty());
    }

    #[test]
    fn sequence_by_checksum_returns_match_and_none() {
        let repo = make_repository();
        assert_eq!(
            repo.sequence_by_checksum("chk-chr1").unwrap().name.as_str(),
            "chr1"
        );
        assert!(repo.sequence_by_checksum("missing").is_none());
    }

    #[test]
    fn gene_returns_record_with_transcripts_and_exons() {
        let repo = make_repository();
        let record = repo.gene(&GeneId::new("Mp1g00010").unwrap()).unwrap();
        assert_eq!(record.gene.id.as_str(), "Mp1g00010");
        assert_eq!(record.transcripts.len(), 1);
        assert_eq!(record.exons.len(), 1);
        assert_eq!(record.cdss.len(), 1);
        assert_eq!(record.cdss[0].region.start.get(), 10);
        assert_eq!(record.cdss[0].region.end.get(), 40);
        assert_eq!(record.cdss[0].phase, Some(0));

        assert!(repo.gene(&GeneId::new("Mp9g99999").unwrap()).is_none());
    }

    #[test]
    fn transcript_returns_some_for_known_id_and_none_otherwise() {
        let repo = make_repository();
        let known = TranscriptId::new("Mp1g00010.1").unwrap();
        let transcript = repo.transcript(&known).unwrap();
        assert_eq!(transcript.id.as_str(), "Mp1g00010.1");
        assert_eq!(transcript.gene_id.as_str(), "Mp1g00010");

        let unknown = TranscriptId::new("Mp9g99999.1").unwrap();
        assert!(repo.transcript(&unknown).is_none());
    }

    #[test]
    fn search_genes_returns_all_when_no_filters() {
        let repo = make_repository();
        let result = repo.search_genes(&GeneSearch::default());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn search_genes_filters_by_symbol() {
        let repo = make_repository();
        let result = repo.search_genes(&GeneSearch {
            symbol: Some("MpFOO".to_owned()),
            ..Default::default()
        });
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.as_str(), "Mp1g00010");
    }

    #[test]
    fn search_genes_filters_by_locus_tag() {
        let repo = make_repository();
        let result = repo.search_genes(&GeneSearch {
            locus_tag: Some("LOCUS2".to_owned()),
            ..Default::default()
        });
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id.as_str(), "Mp1g00020");
    }

    #[test]
    fn search_genes_query_matches_id_symbol_locus_and_attribute() {
        let repo = make_repository();
        let by_id = repo.search_genes(&GeneSearch {
            query: Some("Mp1g00020".to_owned()),
            ..Default::default()
        });
        assert_eq!(by_id.len(), 1);

        let by_symbol = repo.search_genes(&GeneSearch {
            query: Some("MpBAZ".to_owned()),
            ..Default::default()
        });
        assert_eq!(by_symbol.len(), 1);

        let by_locus = repo.search_genes(&GeneSearch {
            query: Some("LOCUS1".to_owned()),
            ..Default::default()
        });
        assert_eq!(by_locus.len(), 1);

        let by_attr = repo.search_genes(&GeneSearch {
            query: Some("interesting-bar".to_owned()),
            ..Default::default()
        });
        assert_eq!(by_attr.len(), 1);

        let no_match = repo.search_genes(&GeneSearch {
            query: Some("does-not-exist".to_owned()),
            ..Default::default()
        });
        assert!(no_match.is_empty());
    }

    #[test]
    fn search_genes_filters_by_tax_id() {
        let repo = make_repository();
        let matching = repo.search_genes(&GeneSearch {
            tax_id: Some(TaxId::new(TAX_ID)),
            ..Default::default()
        });
        assert_eq!(matching.len(), 2);

        let other = repo.search_genes(&GeneSearch {
            tax_id: Some(TaxId::new(9999)),
            ..Default::default()
        });
        assert!(other.is_empty());
    }

    #[test]
    fn features_in_region_returns_overlapping_genes_only() {
        let repo = make_repository();
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let overlapping = repo.features_in_region(&accession, &region(50, 60));
        assert_eq!(overlapping.len(), 1);
        assert_eq!(overlapping[0].id.as_str(), "Mp1g00010");

        let disjoint = repo.features_in_region(&accession, &region(700, 800));
        assert!(disjoint.is_empty());
    }

    #[test]
    fn features_in_region_returns_empty_for_other_assembly() {
        let repo = make_repository();
        let other = AssemblyAccession::new("GCA_other").unwrap();
        assert!(repo.features_in_region(&other, &region(0, 1000)).is_empty());
    }

    fn gene_with_kegg(id: &str, kos: &[&str]) -> Gene {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        Gene {
            id: GeneId::new(id).unwrap(),
            assembly_accession: accession,
            symbol: None,
            locus_tag: None,
            sequence_name: SequenceName::new("chr1").unwrap(),
            region: region(0, 10),
            strand: Strand::Forward,
            feature_type: "gene".to_owned(),
            annotations: kos
                .iter()
                .map(|ko| {
                    FunctionalAnnotation::Kegg(genome_domain::KeggAnnotation::new(
                        KeggEntryId::new(*ko).unwrap(),
                        None,
                        genome_domain::AnnotationEvidence::new(
                            genome_domain::AnnotationSource::Kegg,
                        ),
                    ))
                })
                .collect(),
            attributes: BTreeMap::new(),
        }
    }

    fn repository_with_genes(genes: Vec<Gene>) -> FileGenomeRepository {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        FileGenomeRepository::new(GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(TAX_ID),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession,
                tax_id: TaxId::new(TAX_ID),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: Vec::new(),
            genes,
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_domain::KeggCatalog::default(),
            orthogroup_catalog: genome_domain::OrthogroupCatalog::default(),
        })
    }

    #[test]
    fn genes_with_kegg_ko_groups_by_normalized_ko_and_strips_prefix() {
        // Gene A has both `K00001` (bare) and `ko:K00001` (prefixed) — both
        // should canonicalize to `K00001` and produce one entry, not two.
        // Gene B has a different KO, so it must NOT match K00001.
        let repo = repository_with_genes(vec![
            gene_with_kegg("MpAAA", &["K00001", "ko:K00001"]),
            gene_with_kegg("MpBBB", &["K00002"]),
        ]);

        let hits = repo.genes_with_kegg_ko(&KeggEntryId::new("K00001").unwrap());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id.as_str(), "MpAAA");

        let other = repo.genes_with_kegg_ko(&KeggEntryId::new("K00002").unwrap());
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].id.as_str(), "MpBBB");
    }

    #[test]
    fn genes_with_kegg_ko_ignores_non_orthology_kegg_entries() {
        // A KEGG annotation whose entry id is a pathway (not a K-code) must
        // not be indexed under `ko_entry_id`.
        let repo = repository_with_genes(vec![gene_with_kegg("MpPATH", &["map00010"])]);
        assert!(
            repo.genes_with_kegg_ko(&KeggEntryId::new("map00010").unwrap())
                .is_empty()
        );
    }

    #[test]
    fn genes_with_kegg_ko_returns_empty_for_unknown_ko() {
        let repo = repository_with_genes(vec![gene_with_kegg("MpAAA", &["K00001"])]);
        assert!(
            repo.genes_with_kegg_ko(&KeggEntryId::new("K99999").unwrap())
                .is_empty()
        );
    }

    #[test]
    fn kegg_catalog_returns_the_actual_dataset_catalog() {
        // Build a non-empty catalog so a default-replacement mutation on
        // `kegg_catalog` would visibly differ from the truth.
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let catalog = genome_domain::KeggCatalog {
            pathways: vec![genome_domain::KeggPathway {
                id: genome_domain::KeggPathwayId::new("map00010").unwrap(),
                name: Some("Glycolysis".to_owned()),
            }],
            modules: Vec::new(),
            reactions: Vec::new(),
            ko_links: Vec::new(),
        };
        let repo = FileGenomeRepository::new(GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(TAX_ID),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession,
                tax_id: TaxId::new(TAX_ID),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: Vec::new(),
            genes: Vec::new(),
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: catalog,
            orthogroup_catalog: genome_domain::OrthogroupCatalog::default(),
        });

        let exposed = repo.kegg_catalog();
        assert_eq!(exposed.pathways.len(), 1);
        assert_eq!(exposed.pathways[0].id.as_str(), "map00010");
        assert_eq!(exposed.pathways[0].name.as_deref(), Some("Glycolysis"));
    }

    #[test]
    fn orthogroups_for_gene_returns_sorted_groups_and_unknown_empty() {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let repo = repository_with_orthogroups(vec![
            orthogroup(
                "OG2",
                vec![orthogroup_member(
                    "MpAAA",
                    &accession,
                    "Marchantia polymorpha",
                )],
            ),
            orthogroup(
                "OG1",
                vec![
                    orthogroup_member("AT1G01010", &accession, "Arabidopsis thaliana"),
                    orthogroup_member("MpAAA", &accession, "Marchantia polymorpha"),
                ],
            ),
        ]);

        let hits = repo.orthogroups_for_gene(&GeneId::new("MpAAA").unwrap());

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id.as_str(), "OG1");
        assert_eq!(hits[1].id.as_str(), "OG2");
        assert!(
            repo.orthogroups_for_gene(&GeneId::new("missing").unwrap())
                .is_empty()
        );
    }

    #[test]
    fn orthogroup_returns_group_by_id() {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        let repo = repository_with_orthogroups(vec![orthogroup(
            "OG1",
            vec![orthogroup_member(
                "MpAAA",
                &accession,
                "Marchantia polymorpha",
            )],
        )]);

        let hit = repo.orthogroup(&OrthogroupId::new("OG1").unwrap()).unwrap();

        assert_eq!(hit.id.as_str(), "OG1");
        assert!(
            repo.orthogroup(&OrthogroupId::new("OG404").unwrap())
                .is_none()
        );
    }

    fn orthogroup(id: &str, members: Vec<genome_domain::OrthogroupMember>) -> Orthogroup {
        Orthogroup {
            id: OrthogroupId::new(id).unwrap(),
            members,
        }
    }

    fn orthogroup_member(
        id: &str,
        accession: &AssemblyAccession,
        scientific_name: &str,
    ) -> genome_domain::OrthogroupMember {
        genome_domain::OrthogroupMember {
            gene_id: GeneId::new(id).unwrap(),
            tax_id: TaxId::new(TAX_ID),
            scientific_name: scientific_name.to_owned(),
            assembly_accession: Some(accession.clone()),
            symbol: None,
        }
    }

    fn repository_with_orthogroups(groups: Vec<Orthogroup>) -> FileGenomeRepository {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        FileGenomeRepository::new(GenomeDataset {
            taxon: Taxon {
                tax_id: TaxId::new(TAX_ID),
                scientific_name: "Marchantia polymorpha".to_owned(),
                common_name: None,
                rank: "species".to_owned(),
            },
            assembly: Assembly {
                accession: accession.clone(),
                tax_id: TaxId::new(TAX_ID),
                name: "test".to_owned(),
                source: AssemblySource::Local,
                refget_checksum: None,
            },
            sequences: Vec::new(),
            genes: vec![gene_with_kegg("MpAAA", &[])],
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_domain::KeggCatalog::default(),
            orthogroup_catalog: genome_domain::OrthogroupCatalog { groups },
        })
    }
}
