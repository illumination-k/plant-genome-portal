use std::collections::HashMap;
use std::path::Path;

use genome_core::{
    Assembly, AssemblyAccession, Exon, Gene, GeneId, GeneRecord, GeneSearch, GenomeDataset,
    GenomeRepository, HalfOpenRegion, Sequence, TaxId, Taxon, Transcript, TranscriptId,
};

use crate::error::StorageError;
use crate::snapshot::read_snapshot;

#[derive(Debug, Clone)]
pub struct FileGenomeRepository {
    dataset: GenomeDataset,
    genes_by_id: HashMap<GeneId, Gene>,
    transcripts_by_gene: HashMap<GeneId, Vec<Transcript>>,
    exons_by_transcript: HashMap<TranscriptId, Vec<Exon>>,
    sequence_by_checksum: HashMap<String, Sequence>,
}

impl FileGenomeRepository {
    pub fn new(dataset: GenomeDataset) -> Self {
        let genes_by_id = dataset
            .genes
            .iter()
            .map(|gene| (gene.id.clone(), gene.clone()))
            .collect::<HashMap<_, _>>();

        let mut transcripts_by_gene: HashMap<GeneId, Vec<Transcript>> = HashMap::new();
        for transcript in &dataset.transcripts {
            transcripts_by_gene
                .entry(transcript.gene_id.clone())
                .or_default()
                .push(transcript.clone());
        }

        let mut exons_by_transcript: HashMap<TranscriptId, Vec<Exon>> = HashMap::new();
        for exon in &dataset.exons {
            exons_by_transcript
                .entry(exon.transcript_id.clone())
                .or_default()
                .push(exon.clone());
        }

        let sequence_by_checksum = dataset
            .sequences
            .iter()
            .map(|sequence| (sequence.refget_checksum.clone(), sequence.clone()))
            .collect::<HashMap<_, _>>();

        Self {
            dataset,
            genes_by_id,
            transcripts_by_gene,
            exons_by_transcript,
            sequence_by_checksum,
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
        self.sequence_by_checksum.get(checksum).cloned()
    }

    fn gene(&self, gene_id: &GeneId) -> Option<GeneRecord> {
        let gene = self.genes_by_id.get(gene_id)?.clone();
        let transcripts = self
            .transcripts_by_gene
            .get(gene_id)
            .cloned()
            .unwrap_or_default();
        let exons = transcripts
            .iter()
            .flat_map(|transcript| {
                self.exons_by_transcript
                    .get(&transcript.id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        Some(GeneRecord {
            gene,
            transcripts,
            exons,
        })
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
