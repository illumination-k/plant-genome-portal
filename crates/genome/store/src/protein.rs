use std::path::Path;

use genome_domain::TranscriptId;

use crate::error::StorageError;
use crate::fasta::{read_fasta_sequences, refget_checksum};
use crate::gff::ParsedGff;

/// Parse a protein FASTA file (one record per transcript, header is the
/// transcript id) and attach refget checksums + lengths to the matching
/// transcripts. Records whose header does not match a known transcript are
/// ignored — MarpolBase ships extras (e.g. organellar) that we don't model.
pub(crate) fn apply_protein_sequences(
    parsed: &mut ParsedGff,
    protein_fasta_path: impl AsRef<Path>,
) -> Result<(), StorageError> {
    let proteins = read_fasta_sequences(protein_fasta_path)?;
    for transcript in &mut parsed.transcripts {
        let Ok(key) = TranscriptId::new(transcript.id.as_str()) else {
            continue;
        };
        if let Some(protein) = proteins.get(key.as_str()) {
            let bases = strip_stop_codon(&protein.bases);
            transcript.protein_checksum = Some(refget_checksum(bases.as_bytes()));
            transcript.protein_length = Some(bases.len() as u64);
        }
    }
    Ok(())
}

/// Trim a trailing stop codon (`*`) if the upstream FASTA encodes one.
/// MarpolBase v7.1 sequences do not include a stop, but other sources do, and
/// keeping the checksum stable across both forms is desirable for refget.
fn strip_stop_codon(bases: &str) -> &str {
    bases.strip_suffix('*').unwrap_or(bases)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Write;

    use genome_domain::{
        GeneId, HalfOpenRegion, Position0, SequenceName, Strand, Transcript, TranscriptId,
    };

    use super::*;

    fn transcript(id: &str) -> Transcript {
        Transcript {
            id: TranscriptId::new(id).unwrap(),
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
        }
    }

    #[test]
    fn apply_protein_sequences_attaches_checksum_and_length_to_known_transcripts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("proteins.fa");
        let mut file = File::create(&path).unwrap();
        writeln!(file, ">Mp1g00010.1").unwrap();
        writeln!(file, "MVTAGSMMHL").unwrap();
        writeln!(file, ">Mp1g00020.1").unwrap();
        writeln!(file, "MAAMAAASTA").unwrap();
        drop(file);

        let mut parsed = ParsedGff {
            transcripts: vec![transcript("Mp1g00010.1"), transcript("Mp1g00020.1")],
            ..ParsedGff::default()
        };

        apply_protein_sequences(&mut parsed, &path).unwrap();

        let expected_first = refget_checksum(b"MVTAGSMMHL");
        let expected_second = refget_checksum(b"MAAMAAASTA");
        assert_eq!(
            parsed.transcripts[0].protein_checksum.as_deref(),
            Some(expected_first.as_str())
        );
        assert_eq!(parsed.transcripts[0].protein_length, Some(10));
        assert_eq!(
            parsed.transcripts[1].protein_checksum.as_deref(),
            Some(expected_second.as_str())
        );
        assert_eq!(parsed.transcripts[1].protein_length, Some(10));
    }

    #[test]
    fn apply_protein_sequences_strips_trailing_stop_codon() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("proteins.fa");
        let mut file = File::create(&path).unwrap();
        writeln!(file, ">Mp1g00010.1").unwrap();
        writeln!(file, "MVTAGSMMHL*").unwrap();
        drop(file);

        let mut parsed = ParsedGff {
            transcripts: vec![transcript("Mp1g00010.1")],
            ..ParsedGff::default()
        };

        apply_protein_sequences(&mut parsed, &path).unwrap();
        assert_eq!(parsed.transcripts[0].protein_length, Some(10));
        assert_eq!(
            parsed.transcripts[0].protein_checksum.as_deref(),
            Some(refget_checksum(b"MVTAGSMMHL").as_str())
        );
    }

    #[test]
    fn apply_protein_sequences_ignores_records_with_no_matching_transcript() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("proteins.fa");
        let mut file = File::create(&path).unwrap();
        writeln!(file, ">Mp_unknown.1").unwrap();
        writeln!(file, "MVTAG").unwrap();
        drop(file);

        let mut parsed = ParsedGff {
            transcripts: vec![transcript("Mp1g00010.1")],
            ..ParsedGff::default()
        };

        apply_protein_sequences(&mut parsed, &path).unwrap();
        assert!(parsed.transcripts[0].protein_checksum.is_none());
        assert!(parsed.transcripts[0].protein_length.is_none());
    }
}
