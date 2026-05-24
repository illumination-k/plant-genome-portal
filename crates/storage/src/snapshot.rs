use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use genome_core::{Assembly, GenomeDataset, Sequence, SequenceName, Taxon};
use serde::{Deserialize, Serialize};

use crate::annotation::{apply_functional_annotations, parse_functional_annotations};
use crate::error::StorageError;
use crate::fasta::{assembly_checksum, read_fasta_sequences, refget_checksum};
use crate::gff::{ParsedGff, parse_gff3};
use crate::nomenclature::{apply_nomenclature, parse_nomenclature};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub source_base_url: String,
    pub fasta_file: String,
    pub gff_file: String,
    pub functional_annotation_file: Option<String>,
    pub nomenclature_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenomeSnapshot {
    pub manifest: SnapshotManifest,
    pub dataset: GenomeDataset,
}

pub fn read_snapshot(path: impl AsRef<Path>) -> Result<GenomeSnapshot, StorageError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn write_snapshot(
    path: impl AsRef<Path>,
    snapshot: &GenomeSnapshot,
) -> Result<(), StorageError> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, snapshot)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct GenomeSnapshotBuild {
    pub fasta_path: PathBuf,
    pub gff_path: PathBuf,
    pub functional_annotation_path: Option<PathBuf>,
    pub nomenclature_path: Option<PathBuf>,
    pub manifest: SnapshotManifest,
    pub taxon: Taxon,
    pub assembly: Assembly,
}

pub fn build_genome_snapshot(config: &GenomeSnapshotBuild) -> Result<GenomeSnapshot, StorageError> {
    let sequences = read_fasta_sequences(&config.fasta_path)?;
    let mut parsed_gff = parse_gff3(&config.gff_path, &config.assembly.accession)?;
    enrich_parsed_gff(&mut parsed_gff, config)?;

    let sequence_models = sequences
        .values()
        .map(|sequence| {
            Ok(Sequence {
                name: SequenceName::new(sequence.name.clone())?,
                assembly_accession: config.assembly.accession.clone(),
                length: sequence.bases.len() as u64,
                refget_checksum: refget_checksum(sequence.bases.as_bytes()),
            })
        })
        .collect::<Result<Vec<_>, StorageError>>()?;

    let assembly_checksum = assembly_checksum(&sequence_models);
    let mut assembly = config.assembly.clone();
    assembly.refget_checksum = Some(assembly_checksum);

    Ok(GenomeSnapshot {
        manifest: config.manifest.clone(),
        dataset: GenomeDataset {
            taxon: config.taxon.clone(),
            assembly,
            sequences: sequence_models,
            genes: parsed_gff.genes,
            transcripts: parsed_gff.transcripts,
            exons: parsed_gff.exons,
        },
    })
}

fn enrich_parsed_gff(
    parsed: &mut ParsedGff,
    config: &GenomeSnapshotBuild,
) -> Result<(), StorageError> {
    if let Some(path) = &config.functional_annotation_path {
        apply_functional_annotations(parsed, &parse_functional_annotations(path)?);
    }
    if let Some(path) = &config.nomenclature_path {
        apply_nomenclature(parsed, &parse_nomenclature(path)?);
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use genome_core::{Assembly, AssemblyAccession, AssemblySource, GenomeDataset, TaxId, Taxon};

    use super::*;

    fn sample_snapshot() -> GenomeSnapshot {
        GenomeSnapshot {
            manifest: SnapshotManifest {
                source_base_url: "https://example.test".to_owned(),
                fasta_file: "test.fa".to_owned(),
                gff_file: "test.gff".to_owned(),
                functional_annotation_file: None,
                nomenclature_file: None,
            },
            dataset: GenomeDataset {
                taxon: Taxon {
                    tax_id: TaxId::new(3197),
                    scientific_name: "Marchantia polymorpha".to_owned(),
                    common_name: None,
                    rank: "species".to_owned(),
                },
                assembly: Assembly {
                    accession: AssemblyAccession::new("GCA_test").unwrap(),
                    tax_id: TaxId::new(3197),
                    name: "test".to_owned(),
                    source: AssemblySource::Local,
                    refget_checksum: None,
                },
                sequences: Vec::new(),
                genes: Vec::new(),
                transcripts: Vec::new(),
                exons: Vec::new(),
            },
        }
    }

    #[test]
    fn write_snapshot_then_read_snapshot_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        let snapshot = sample_snapshot();

        write_snapshot(&path, &snapshot).unwrap();
        assert!(path.exists());
        let bytes = std::fs::metadata(&path).unwrap().len();
        assert!(bytes > 0, "write_snapshot must write a non-empty file");

        let loaded = read_snapshot(&path).unwrap();
        assert_eq!(loaded, snapshot);
    }
}
