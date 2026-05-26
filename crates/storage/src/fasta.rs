use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use base64::Engine;
use flate2::read::GzDecoder;
use genome_core::Sequence;
use noodles_fasta as fasta;
use sha2::{Digest, Sha512};

use crate::error::StorageError;

#[derive(Debug, Clone)]
pub(crate) struct FastaSequence {
    pub name: String,
    pub bases: String,
}

pub(crate) fn read_fasta_sequences(
    path: impl AsRef<Path>,
) -> Result<HashMap<String, FastaSequence>, StorageError> {
    let path = path.as_ref();
    let reader = open_maybe_gzip(path)?;
    let mut reader = fasta::io::Reader::new(BufReader::new(reader));
    let mut sequences = HashMap::new();

    for result in reader.records() {
        let record = result?;
        let name = std::str::from_utf8(record.name())
            .map_err(|_| StorageError::InvalidFastaRecord(path.to_path_buf()))?
            .to_owned();
        let bases = bases_to_uppercase_string(record.sequence().as_ref());
        sequences.insert(name.clone(), FastaSequence { name, bases });
    }

    Ok(sequences)
}

fn bases_to_uppercase_string(bases: &[u8]) -> String {
    let mut out = String::with_capacity(bases.len());
    for &byte in bases {
        if byte.is_ascii_whitespace() {
            continue;
        }
        out.push(byte.to_ascii_uppercase() as char);
    }
    out
}

fn open_maybe_gzip(path: &Path) -> Result<Box<dyn Read>, StorageError> {
    let file = File::open(path)?;
    if path.extension().is_some_and(|extension| extension == "gz") {
        Ok(Box::new(GzDecoder::new(file)))
    } else {
        Ok(Box::new(file))
    }
}

#[derive(Debug, Clone)]
pub struct FastaReference {
    by_checksum: HashMap<String, String>,
}

impl FastaReference {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let mut reference = Self {
            by_checksum: HashMap::new(),
        };
        reference.extend_from_path(path)?;
        Ok(reference)
    }

    /// Load an additional FASTA file (e.g. a protein FASTA) into the same
    /// checksum-keyed index. Records that share a refget checksum with an
    /// already-loaded sequence are silently de-duplicated.
    pub fn extend_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        for sequence in read_fasta_sequences(path)?.into_values() {
            self.by_checksum
                .entry(refget_checksum(sequence.bases.as_bytes()))
                .or_insert(sequence.bases);
        }
        Ok(())
    }

    pub fn get(&self, checksum: &str, start: Option<u64>, end: Option<u64>) -> Option<String> {
        let sequence = self.by_checksum.get(checksum)?;
        let len = sequence.len() as u64;
        let start = start.unwrap_or(0).min(len);
        let end = end.unwrap_or(len).min(len);
        if start > end {
            return None;
        }
        sequence
            .get(start as usize..end as usize)
            .map(str::to_owned)
    }
}

pub fn refget_checksum(sequence: &[u8]) -> String {
    let mut digest = Sha512::new();
    for base in sequence {
        if base.is_ascii_whitespace() {
            continue;
        }
        digest.update([base.to_ascii_uppercase()]);
    }
    let digest = digest.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..24])
}

pub(crate) fn assembly_checksum(sequences: &[Sequence]) -> String {
    let mut checksums = sequences
        .iter()
        .map(|sequence| sequence.refget_checksum.as_str())
        .collect::<Vec<_>>();
    checksums.sort_unstable();

    let mut digest = Sha512::new();
    for checksum in checksums {
        digest.update(checksum.as_bytes());
        digest.update(b"\n");
    }

    let digest = digest.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..24])
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use genome_core::{AssemblyAccession, SequenceName};

    use super::*;

    #[test]
    fn refget_checksum_is_case_and_line_invariant() {
        assert_eq!(refget_checksum(b"acgtn\n"), refget_checksum(b"ACGTN"));
    }

    #[test]
    fn refget_checksum_value_depends_on_input() {
        let checksum = refget_checksum(b"ACGT");
        assert_eq!(checksum.len(), 32);
        assert_ne!(checksum, refget_checksum(b"TGCA"));
        assert!(!checksum.is_empty());
    }

    fn make_reference() -> FastaReference {
        let mut by_checksum = HashMap::new();
        by_checksum.insert(refget_checksum(b"ACGTACGT"), "ACGTACGT".to_owned());
        FastaReference { by_checksum }
    }

    #[test]
    fn get_returns_full_sequence_when_bounds_are_unset() {
        let reference = make_reference();
        let checksum = refget_checksum(b"ACGTACGT");
        assert_eq!(
            reference.get(&checksum, None, None).as_deref(),
            Some("ACGTACGT")
        );
    }

    #[test]
    fn get_returns_requested_substring() {
        let reference = make_reference();
        let checksum = refget_checksum(b"ACGTACGT");
        assert_eq!(
            reference.get(&checksum, Some(2), Some(6)).as_deref(),
            Some("GTAC")
        );
    }

    #[test]
    fn get_returns_none_for_unknown_checksum() {
        let reference = make_reference();
        assert_eq!(reference.get("missing", None, None), None);
    }

    #[test]
    fn get_returns_empty_string_when_start_equals_end() {
        let reference = make_reference();
        let checksum = refget_checksum(b"ACGTACGT");
        assert_eq!(
            reference.get(&checksum, Some(3), Some(3)).as_deref(),
            Some("")
        );
    }

    #[test]
    fn get_returns_none_when_start_after_end() {
        let reference = make_reference();
        let checksum = refget_checksum(b"ACGTACGT");
        assert_eq!(reference.get(&checksum, Some(5), Some(3)), None);
    }

    #[test]
    fn get_clamps_end_beyond_length() {
        let reference = make_reference();
        let checksum = refget_checksum(b"ACGTACGT");
        assert_eq!(
            reference.get(&checksum, Some(4), Some(100)).as_deref(),
            Some("ACGT")
        );
    }

    #[test]
    fn extend_from_path_adds_new_sequences_by_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let primary = dir.path().join("primary.fa");
        std::fs::write(&primary, b">chrA\nACGT\n").unwrap();
        let secondary = dir.path().join("secondary.fa");
        std::fs::write(&secondary, b">protA\nMVTAG\n").unwrap();

        let mut reference = FastaReference::from_path(&primary).unwrap();
        let checksum_protein = refget_checksum(b"MVTAG");
        assert!(reference.get(&checksum_protein, None, None).is_none());

        reference.extend_from_path(&secondary).unwrap();
        assert_eq!(
            reference.get(&checksum_protein, None, None).as_deref(),
            Some("MVTAG")
        );
        let checksum_dna = refget_checksum(b"ACGT");
        assert_eq!(
            reference.get(&checksum_dna, None, None).as_deref(),
            Some("ACGT")
        );
    }

    fn make_sequence(name: &str, checksum: &str) -> genome_core::Sequence {
        genome_core::Sequence {
            name: SequenceName::new(name).unwrap(),
            assembly_accession: AssemblyAccession::new("GCA_test").unwrap(),
            length: 8,
            refget_checksum: checksum.to_owned(),
        }
    }

    #[test]
    fn assembly_checksum_is_deterministic_and_order_independent() {
        let a = make_sequence("chr1", "aaaa");
        let b = make_sequence("chr2", "bbbb");
        let forward = assembly_checksum(&[a.clone(), b.clone()]);
        let reverse = assembly_checksum(&[b, a]);
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 32);
    }

    #[test]
    fn assembly_checksum_differs_with_membership() {
        let a = make_sequence("chr1", "aaaa");
        let b = make_sequence("chr2", "bbbb");
        let with_both = assembly_checksum(&[a.clone(), b]);
        let with_one = assembly_checksum(&[a]);
        assert_ne!(with_both, with_one);
    }
}
