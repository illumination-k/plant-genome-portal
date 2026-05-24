use genome_core::{
    AssemblyAccession, GeneId, GenomeRepository, HalfOpenRegion, HomologyHit, HomologySearchResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub trait HomologyAnnotationRepository: Send + Sync + 'static {
    fn overlapping_gene_ids(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<GeneId>;
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
        AnnotatedHomologySearchResult {
            method: result.method,
            task: result.task,
            hits: result
                .hits
                .into_iter()
                .map(|hit| self.annotate_hit(hit))
                .collect(),
        }
    }

    fn annotate_hit(&self, hit: HomologyHit) -> AnnotatedHomologyHit {
        let overlapping_gene_ids = hit
            .subject_region
            .to_half_open()
            .map(|region| {
                self.repository
                    .overlapping_gene_ids(&hit.assembly_accession, &region)
            })
            .unwrap_or_default();

        AnnotatedHomologyHit {
            hit,
            overlapping_gene_ids,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotatedHomologySearchResult {
    pub method: genome_core::HomologySearchMethod,
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
    use genome_core::{
        Assembly, AssemblySource, GenomeDataset, HomologySearchMethod, Position0, Position1,
        SequenceName, Strand, TaxId,
    };
    use std::collections::BTreeMap;
    use storage::FileGenomeRepository;

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
            taxon: genome_core::Taxon {
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
            genes: vec![genome_core::Gene {
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
        })
    }
}
