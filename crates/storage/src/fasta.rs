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
        let by_checksum = read_fasta_sequences(path)?
            .into_values()
            .map(|sequence| (refget_checksum(sequence.bases.as_bytes()), sequence.bases))
            .collect::<HashMap<_, _>>();
        Ok(Self { by_checksum })
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
    use super::*;

    #[test]
    fn refget_checksum_is_case_and_line_invariant() {
        assert_eq!(refget_checksum(b"acgtn\n"), refget_checksum(b"ACGTN"));
    }
}
