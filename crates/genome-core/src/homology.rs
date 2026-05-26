use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::coord::{ClosedRegion, Position1, Strand};
use crate::error::DomainError;
use crate::ids::{AssemblyAccession, SequenceName, TranscriptId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HomologySearchMethod {
    Blastn,
    Blastp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomologySearchResult {
    pub method: HomologySearchMethod,
    pub task: String,
    pub hits: Vec<HomologyHit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomologyHit {
    pub query_id: String,
    pub assembly_accession: AssemblyAccession,
    pub sequence_name: SequenceName,
    pub subject_region: ClosedRegion,
    pub strand: Strand,
    pub percent_identity: f64,
    pub alignment_length: u64,
    pub mismatches: u64,
    pub gap_opens: u64,
    pub query_start: Position1,
    pub query_end: Position1,
    pub subject_start: Position1,
    pub subject_end: Position1,
    pub evalue: f64,
    pub bit_score: f64,
    pub query_alignment: String,
    pub subject_alignment: String,
}

impl HomologyHit {
    /// Build a blastp hit. The subject is a transcript (protein) identifier,
    /// not a genomic sequence — `sequence_name` carries the transcript id and
    /// `subject_region` describes amino-acid positions on that protein. Protein
    /// alignments are non-stranded; we record `Strand::Unknown`.
    #[allow(clippy::too_many_arguments)]
    pub fn from_blastp_alignment(
        assembly_accession: AssemblyAccession,
        query_id: String,
        subject_transcript_id: TranscriptId,
        percent_identity: f64,
        alignment_length: u64,
        mismatches: u64,
        gap_opens: u64,
        query_start: Position1,
        query_end: Position1,
        subject_start: Position1,
        subject_end: Position1,
        evalue: f64,
        bit_score: f64,
        query_alignment: String,
        subject_alignment: String,
    ) -> Result<Self, DomainError> {
        let sequence_name = SequenceName::new(subject_transcript_id.as_str())?;
        let region_start = subject_start.min(subject_end);
        let region_end = subject_start.max(subject_end);
        let subject_region = ClosedRegion::new(sequence_name.clone(), region_start, region_end)?;

        Ok(Self {
            query_id,
            assembly_accession,
            sequence_name,
            subject_region,
            strand: Strand::Unknown,
            percent_identity,
            alignment_length,
            mismatches,
            gap_opens,
            query_start,
            query_end,
            subject_start,
            subject_end,
            evalue,
            bit_score,
            query_alignment,
            subject_alignment,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_blastn_alignment(
        assembly_accession: AssemblyAccession,
        query_id: String,
        sequence_name: SequenceName,
        percent_identity: f64,
        alignment_length: u64,
        mismatches: u64,
        gap_opens: u64,
        query_start: Position1,
        query_end: Position1,
        subject_start: Position1,
        subject_end: Position1,
        evalue: f64,
        bit_score: f64,
        query_alignment: String,
        subject_alignment: String,
    ) -> Result<Self, DomainError> {
        let strand = if subject_start <= subject_end {
            Strand::Forward
        } else {
            Strand::Reverse
        };
        let region_start = subject_start.min(subject_end);
        let region_end = subject_start.max(subject_end);
        let subject_region = ClosedRegion::new(sequence_name.clone(), region_start, region_end)?;

        Ok(Self {
            query_id,
            assembly_accession,
            sequence_name,
            subject_region,
            strand,
            percent_identity,
            alignment_length,
            mismatches,
            gap_opens,
            query_start,
            query_end,
            subject_start,
            subject_end,
            evalue,
            bit_score,
            query_alignment,
            subject_alignment,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn blastn_alignment_normalizes_reverse_subject_region() {
        let hit = HomologyHit::from_blastn_alignment(
            AssemblyAccession::new("GCA_test").unwrap(),
            "query".to_owned(),
            SequenceName::new("chr1").unwrap(),
            99.0,
            100,
            1,
            0,
            Position1::new(1).unwrap(),
            Position1::new(100).unwrap(),
            Position1::new(500).unwrap(),
            Position1::new(401).unwrap(),
            1e-20,
            80.0,
            "ACGT".to_owned(),
            "ACGT".to_owned(),
        )
        .unwrap();

        assert_eq!(hit.strand, Strand::Reverse);
        assert_eq!(hit.subject_region.start.get(), 401);
        assert_eq!(hit.subject_region.end.get(), 500);
        assert_eq!(hit.subject_start.get(), 500);
        assert_eq!(hit.subject_end.get(), 401);
    }
}
