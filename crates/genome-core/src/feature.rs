use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use crate::annotation::FunctionalAnnotation;
use crate::assembly::{Assembly, Sequence, Taxon};
use crate::coord::{HalfOpenRegion, Strand};
use crate::ids::{AssemblyAccession, GeneId, SequenceName, TranscriptId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Gene {
    pub id: GeneId,
    pub assembly_accession: AssemblyAccession,
    pub symbol: Option<String>,
    pub locus_tag: Option<String>,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
    pub feature_type: String,
    pub annotations: Vec<FunctionalAnnotation>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Transcript {
    pub id: TranscriptId,
    pub gene_id: GeneId,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
    pub feature_type: String,
    pub annotations: Vec<FunctionalAnnotation>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Exon {
    pub transcript_id: TranscriptId,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GeneRecord {
    pub gene: Gene,
    pub transcripts: Vec<Transcript>,
    pub exons: Vec<Exon>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GenomeDataset {
    pub taxon: Taxon,
    pub assembly: Assembly,
    pub sequences: Vec<Sequence>,
    pub genes: Vec<Gene>,
    pub transcripts: Vec<Transcript>,
    pub exons: Vec<Exon>,
}
