use genome_core::{AssemblyAccession, GenomeRepository, HalfOpenRegion, SequenceName, Strand};

use crate::{GenomeService, ServiceError};

pub struct TranscriptProtein {
    pub transcript_id: String,
    pub checksum: String,
    pub sequence: String,
}

impl<R> GenomeService<R>
where
    R: GenomeRepository,
{
    /// Return the translated protein for a transcript, looked up via the
    /// transcript's `protein_checksum` against the configured FASTA reference.
    pub fn transcript_protein(
        &self,
        transcript_id: &str,
    ) -> Result<TranscriptProtein, ServiceError> {
        let transcript = self.transcript(transcript_id)?;
        let checksum = transcript
            .protein_checksum
            .clone()
            .ok_or_else(|| ServiceError::ProteinSequenceUnavailable(transcript.id.to_string()))?;
        let reference = self
            .reference
            .as_ref()
            .ok_or_else(|| ServiceError::ProteinSequenceUnavailable(transcript.id.to_string()))?;
        let sequence = reference
            .get(&checksum, None, None)
            .ok_or_else(|| ServiceError::ProteinSequenceUnavailable(transcript.id.to_string()))?;
        Ok(TranscriptProtein {
            transcript_id: transcript.id.into_string(),
            checksum,
            sequence,
        })
    }
}

impl<R> GenomeService<R>
where
    R: GenomeRepository,
{
    pub fn sequence_segments_for_assembly(
        &self,
        accession: &str,
        sequence_name: &str,
        segments: Vec<HalfOpenRegion>,
        strand: Strand,
    ) -> Result<String, ServiceError> {
        if segments.is_empty() {
            return Err(ServiceError::InvalidRequest(
                "at least one sequence segment is required".to_owned(),
            ));
        }

        let accession = AssemblyAccession::new(accession)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        if self.repository.assembly(&accession).is_none() {
            return Err(ServiceError::AssemblyNotFound(accession.into_string()));
        }

        let sequence_name = SequenceName::new(sequence_name)
            .map_err(|error| ServiceError::InvalidRequest(error.to_string()))?;
        let sequence = self
            .repository
            .sequences_for_assembly(&accession)
            .into_iter()
            .find(|sequence| sequence.name == sequence_name)
            .ok_or_else(|| ServiceError::SequenceNotFound(sequence_name.clone().into_string()))?;

        for segment in &segments {
            if segment.sequence_name != sequence_name {
                return Err(ServiceError::InvalidRequest(
                    "all segments must use the requested sequence name".to_owned(),
                ));
            }
            if segment.end.get() > sequence.length {
                return Err(ServiceError::InvalidRequest(
                    "sequence segment is out of bounds".to_owned(),
                ));
            }
        }

        let reference = self
            .reference
            .as_ref()
            .ok_or_else(|| ServiceError::SequenceNotFound(sequence.refget_checksum.clone()))?;
        let mut joined = String::new();
        for segment in segments {
            let segment_sequence = reference
                .get(
                    &sequence.refget_checksum,
                    Some(segment.start.get()),
                    Some(segment.end.get()),
                )
                .ok_or_else(|| ServiceError::InvalidRequest("invalid sequence range".to_owned()))?;
            joined.push_str(&segment_sequence);
        }

        Ok(match strand {
            Strand::Reverse => reverse_complement(&joined),
            Strand::Forward | Strand::Unknown => joined,
        })
    }
}

fn reverse_complement(sequence: &str) -> String {
    sequence
        .bytes()
        .rev()
        .map(complement_base)
        .map(char::from)
        .collect()
}

fn complement_base(base: u8) -> u8 {
    match base.to_ascii_uppercase() {
        b'A' => b'T',
        b'C' => b'G',
        b'G' => b'C',
        b'T' | b'U' => b'A',
        b'R' => b'Y',
        b'Y' => b'R',
        b'S' => b'S',
        b'W' => b'W',
        b'K' => b'M',
        b'M' => b'K',
        b'B' => b'V',
        b'D' => b'H',
        b'H' => b'D',
        b'V' => b'B',
        b'N' => b'N',
        _ => b'N',
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use genome_core::{Assembly, AssemblySource, GenomeDataset, Position0, Sequence, TaxId, Taxon};
    use std::fs;
    use storage::{FastaReference, FileGenomeRepository, refget_checksum};

    #[test]
    fn sequence_segments_are_concatenated_in_request_order() {
        let service = make_service();

        let sequence = service
            .sequence_segments_for_assembly(
                "GCA_test",
                "chr1",
                vec![segment(0, 4), segment(8, 12)],
                Strand::Forward,
            )
            .unwrap();

        assert_eq!(sequence, "ACGTACGT");
    }

    #[test]
    fn reverse_strand_returns_reverse_complement_of_joined_segments() {
        let service = make_service();

        let sequence = service
            .sequence_segments_for_assembly(
                "GCA_test",
                "chr1",
                vec![segment(0, 4), segment(8, 12)],
                Strand::Reverse,
            )
            .unwrap();

        assert_eq!(sequence, "ACGTACGT");
    }

    #[test]
    fn sequence_segments_reject_out_of_bounds_ranges() {
        let service = make_service();

        let error = service
            .sequence_segments_for_assembly(
                "GCA_test",
                "chr1",
                vec![segment(10, 17)],
                Strand::Forward,
            )
            .unwrap_err();

        assert!(matches!(error, ServiceError::InvalidRequest(_)));
    }

    fn make_service() -> GenomeService<FileGenomeRepository> {
        let bases = b"ACGTNNNNACGT";
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
                assembly_accession: accession,
                length: bases.len() as u64,
                refget_checksum: refget_checksum(bases),
            }],
            genes: Vec::new(),
            transcripts: Vec::new(),
            exons: Vec::new(),
            cdss: Vec::new(),
            kegg_catalog: genome_core::KeggCatalog::default(),
            orthogroup_catalog: genome_core::OrthogroupCatalog::default(),
        };

        static NEXT_FASTA_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let suffix = NEXT_FASTA_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let fasta_path = std::env::temp_dir().join(format!(
            "plant-genome-portal-service-sequence-test-{}-{suffix}.fa",
            std::process::id(),
        ));
        fs::write(&fasta_path, b">chr1\nACGTNNNNACGT\n").unwrap();
        let reference = FastaReference::from_path(&fasta_path).unwrap();
        fs::remove_file(&fasta_path).unwrap();

        GenomeService::new(FileGenomeRepository::new(dataset), Some(reference))
    }

    fn segment(start: u64, end: u64) -> HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(start),
            Position0::new(end),
        )
        .unwrap()
    }
}
