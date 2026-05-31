use genome_domain::{
    AssemblyAccession, GeneId, GenomeRepository, HalfOpenRegion, HomologyHit, HomologySearchMethod,
    HomologySearchResult, TranscriptId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub trait HomologyAnnotationRepository: Send + Sync + 'static {
    fn overlapping_gene_ids(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<GeneId>;

    /// Resolve a transcript subject id to its parent gene id (used to annotate
    /// blastp hits, whose subject is a protein not a genomic region).
    fn gene_id_for_transcript(&self, transcript_id: &TranscriptId) -> Option<GeneId>;
}

impl<T> HomologyAnnotationRepository for T
where
    T: GenomeRepository,
{
    fn overlapping_gene_ids(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<GeneId> {
        self.features_in_region(accession, region)
            .into_iter()
            .map(|gene| gene.id)
            .collect()
    }

    fn gene_id_for_transcript(&self, transcript_id: &TranscriptId) -> Option<GeneId> {
        self.transcript(transcript_id)
            .map(|transcript| transcript.gene_id)
    }
}

#[derive(Clone)]
pub struct HomologyService<R> {
    repository: Arc<R>,
}

impl<R> HomologyService<R>
where
    R: HomologyAnnotationRepository,
{
    pub fn new(repository: R) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub fn annotate_result(&self, result: HomologySearchResult) -> AnnotatedHomologySearchResult {
        let method = result.method.clone();
        AnnotatedHomologySearchResult {
            method: result.method,
            task: result.task,
            hits: result
                .hits
                .into_iter()
                .map(|hit| self.annotate_hit(hit, &method))
                .collect(),
        }
    }

    fn annotate_hit(
        &self,
        hit: HomologyHit,
        method: &HomologySearchMethod,
    ) -> AnnotatedHomologyHit {
        let overlapping_gene_ids = match method {
            HomologySearchMethod::Blastp => self.annotate_blastp_hit(&hit),
            HomologySearchMethod::Blastn => self.annotate_blastn_hit(&hit),
        };
        AnnotatedHomologyHit {
            hit,
            overlapping_gene_ids,
        }
    }

    fn annotate_blastn_hit(&self, hit: &HomologyHit) -> Vec<GeneId> {
        hit.subject_region
            .clone()
            .to_half_open()
            .map(|region| {
                self.repository
                    .overlapping_gene_ids(&hit.assembly_accession, &region)
            })
            .unwrap_or_default()
    }

    fn annotate_blastp_hit(&self, hit: &HomologyHit) -> Vec<GeneId> {
        TranscriptId::new(hit.sequence_name.as_str())
            .ok()
            .and_then(|transcript_id| self.repository.gene_id_for_transcript(&transcript_id))
            .map(|gene_id| vec![gene_id])
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotatedHomologySearchResult {
    pub method: genome_domain::HomologySearchMethod,
    pub task: String,
    pub hits: Vec<AnnotatedHomologyHit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotatedHomologyHit {
    pub hit: HomologyHit,
    pub overlapping_gene_ids: Vec<GeneId>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use genome_domain::{
        Assembly, AssemblySource, GenomeDataset, HomologySearchMethod, Position0, Position1,
        SequenceName, Strand, TaxId,
    };
    use genome_store::FileGenomeRepository;
    use std::collections::BTreeMap;

    #[test]
    fn annotate_result_links_overlapping_gene_ids() {
        let service = HomologyService::new(make_repository());
        let hit = HomologyHit::from_blastn_alignment(
            AssemblyAccession::new("GCA_test").unwrap(),
            "query".to_owned(),
            SequenceName::new("chr1").unwrap(),
            100.0,
            8,
            0,
            0,
            Position1::new(1).unwrap(),
            Position1::new(8).unwrap(),
            Position1::new(2).unwrap(),
            Position1::new(9).unwrap(),
            1e-10,
            42.0,
            "ACGT".to_owned(),
            "ACGT".to_owned(),
        )
        .unwrap();

        let result = service.annotate_result(HomologySearchResult {
            method: HomologySearchMethod::Blastn,
            task: "blastn".to_owned(),
            hits: vec![hit],
        });

        assert_eq!(
            result.hits[0].overlapping_gene_ids,
            vec![GeneId::new("Mp1g00010").unwrap()]
        );
    }

    fn make_repository() -> FileGenomeRepository {
        let accession = AssemblyAccession::new("GCA_test").unwrap();
        FileGenomeRepository::new(GenomeDataset {
            taxon: genome_domain::Taxon {
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
            genes: vec![genome_domain::Gene {
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
            transcripts: vec![genome_domain::Transcript {
                id: genome_domain::TranscriptId::new("Mp1g00010.1").unwrap(),
                gene_id: GeneId::new("Mp1g00010").unwrap(),
                sequence_name: SequenceName::new("chr1").unwrap(),
                region: HalfOpenRegion::new(
                    SequenceName::new("chr1").unwrap(),
                    Position0::new(0),
                    Position0::new(10),
                )
                .unwrap(),
                strand: Strand::Forward,
                feature_type: "mRNA".to_owned(),
                annotations: Vec::new(),
                attributes: BTreeMap::new(),
                protein_checksum: None,
                protein_length: None,
            }],
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_domain::KeggCatalog::default(),
            orthogroup_catalog: genome_domain::OrthogroupCatalog::default(),
        })
    }

    #[test]
    fn annotate_result_resolves_blastp_subject_transcript_to_gene_id() {
        let service = HomologyService::new(make_repository());
        let hit = HomologyHit::from_blastp_alignment(
            AssemblyAccession::new("GCA_test").unwrap(),
            "query".to_owned(),
            genome_domain::TranscriptId::new("Mp1g00010.1").unwrap(),
            100.0,
            120,
            0,
            0,
            Position1::new(1).unwrap(),
            Position1::new(120).unwrap(),
            Position1::new(1).unwrap(),
            Position1::new(120).unwrap(),
            1e-50,
            240.0,
            "MVTAGSMMHL".to_owned(),
            "MVTAGSMMHL".to_owned(),
        )
        .unwrap();

        let result = service.annotate_result(HomologySearchResult {
            method: HomologySearchMethod::Blastp,
            task: "blastp".to_owned(),
            hits: vec![hit],
        });

        assert_eq!(
            result.hits[0].overlapping_gene_ids,
            vec![GeneId::new("Mp1g00010").unwrap()]
        );
    }

    #[test]
    fn annotate_result_returns_no_genes_for_blastp_hit_against_unknown_transcript() {
        let service = HomologyService::new(make_repository());
        let hit = HomologyHit::from_blastp_alignment(
            AssemblyAccession::new("GCA_test").unwrap(),
            "query".to_owned(),
            genome_domain::TranscriptId::new("Mp_unknown.1").unwrap(),
            100.0,
            50,
            0,
            0,
            Position1::new(1).unwrap(),
            Position1::new(50).unwrap(),
            Position1::new(1).unwrap(),
            Position1::new(50).unwrap(),
            1e-20,
            100.0,
            "MVTAG".to_owned(),
            "MVTAG".to_owned(),
        )
        .unwrap();

        let result = service.annotate_result(HomologySearchResult {
            method: HomologySearchMethod::Blastp,
            task: "blastp".to_owned(),
            hits: vec![hit],
        });

        assert!(result.hits[0].overlapping_gene_ids.is_empty());
    }
}
