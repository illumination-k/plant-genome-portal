use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use genome_core::{
    Assembly, GeneId, GenomeDataset, KeggCatalog, OrthogroupCatalog, Sequence, SequenceName, Taxon,
};
use serde::{Deserialize, Serialize};

use crate::annotation::{apply_functional_annotations, parse_functional_annotations};
use crate::error::StorageError;
use crate::fasta::{assembly_checksum, read_fasta_sequences, refget_checksum};
use crate::gff::{ParsedGff, parse_gff3};
use crate::kegg::{KeggCatalogInput, build_kegg_catalog};
use crate::nomenclature::{apply_nomenclature, parse_nomenclature};
use crate::protein::apply_protein_sequences;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub source_base_url: String,
    pub fasta_file: String,
    pub gff_file: String,
    pub functional_annotation_file: Option<String>,
    pub nomenclature_file: Option<String>,
    #[serde(default)]
    pub kegg_files: Option<KeggManifest>,
    #[serde(default)]
    pub protein_fasta_file: Option<String>,
    #[serde(default)]
    pub orthogroup_file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeggManifest {
    pub source_base_url: String,
    pub link_ko_pathway: Option<String>,
    pub link_ko_module: Option<String>,
    pub link_ko_reaction: Option<String>,
    pub list_pathway: Option<String>,
    pub list_module: Option<String>,
    pub list_reaction: Option<String>,
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

#[derive(Debug, Clone, Default)]
pub struct KeggCatalogPaths {
    pub link_ko_pathway: Option<PathBuf>,
    pub link_ko_module: Option<PathBuf>,
    pub link_ko_reaction: Option<PathBuf>,
    pub list_pathway: Option<PathBuf>,
    pub list_module: Option<PathBuf>,
    pub list_reaction: Option<PathBuf>,
}

impl KeggCatalogPaths {
    pub fn is_empty(&self) -> bool {
        self.link_ko_pathway.is_none()
            && self.link_ko_module.is_none()
            && self.link_ko_reaction.is_none()
            && self.list_pathway.is_none()
            && self.list_module.is_none()
            && self.list_reaction.is_none()
    }

    fn as_input(&self) -> KeggCatalogInput<'_> {
        KeggCatalogInput {
            link_ko_pathway: self.link_ko_pathway.as_deref(),
            link_ko_module: self.link_ko_module.as_deref(),
            link_ko_reaction: self.link_ko_reaction.as_deref(),
            list_pathway: self.list_pathway.as_deref(),
            list_module: self.list_module.as_deref(),
            list_reaction: self.list_reaction.as_deref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GenomeSnapshotBuild {
    pub fasta_path: PathBuf,
    pub gff_path: PathBuf,
    pub functional_annotation_path: Option<PathBuf>,
    pub nomenclature_path: Option<PathBuf>,
    pub protein_fasta_path: Option<PathBuf>,
    pub orthogroup_path: Option<PathBuf>,
    pub kegg_catalog_paths: KeggCatalogPaths,
    pub manifest: SnapshotManifest,
    pub taxon: Taxon,
    pub assembly: Assembly,
}

pub fn build_genome_snapshot(config: &GenomeSnapshotBuild) -> Result<GenomeSnapshot, StorageError> {
    let sequences = read_fasta_sequences(&config.fasta_path)?;
    let mut parsed_gff = parse_gff3(&config.gff_path, &config.assembly.accession)?;
    enrich_parsed_gff(&mut parsed_gff, config)?;

    let kegg_catalog = build_optional_kegg_catalog(&parsed_gff, &config.kegg_catalog_paths)?;
    let orthogroup_catalog = build_optional_orthogroup_catalog(config, &parsed_gff)?;
    let sequence_models = build_sequence_models(config, sequences.values())?;
    let assembly = assembly_with_refget_checksum(&config.assembly, &sequence_models);

    Ok(GenomeSnapshot {
        manifest: config.manifest.clone(),
        dataset: GenomeDataset {
            taxon: config.taxon.clone(),
            assembly,
            sequences: sequence_models,
            genes: parsed_gff.genes,
            transcripts: parsed_gff.transcripts,
            exons: parsed_gff.exons,
            cdss: parsed_gff.cdss,
            kegg_catalog,
            orthogroup_catalog,
        },
    })
}

fn build_optional_orthogroup_catalog(
    config: &GenomeSnapshotBuild,
    parsed_gff: &ParsedGff,
) -> Result<OrthogroupCatalog, StorageError> {
    let Some(path) = &config.orthogroup_path else {
        return Ok(OrthogroupCatalog::default());
    };
    let catalog = crate::orthogroup::parse_orthogroups(path)?;
    validate_current_assembly_orthogroup_members(config, parsed_gff, &catalog)?;
    Ok(catalog)
}

fn validate_current_assembly_orthogroup_members(
    config: &GenomeSnapshotBuild,
    parsed_gff: &ParsedGff,
    catalog: &OrthogroupCatalog,
) -> Result<(), StorageError> {
    let gene_ids = parsed_gff
        .genes
        .iter()
        .map(|gene| gene.id.clone())
        .collect::<HashSet<GeneId>>();
    for group in &catalog.groups {
        for member in &group.members {
            if member
                .assembly_accession
                .as_ref()
                .is_some_and(|accession| accession == &config.assembly.accession)
                && !gene_ids.contains(&member.gene_id)
            {
                return Err(StorageError::InvalidTsvValue {
                    line: 0,
                    message: format!(
                        "orthogroup {} references missing current-assembly gene {}",
                        group.id, member.gene_id
                    ),
                });
            }
        }
    }
    Ok(())
}

fn build_optional_kegg_catalog(
    parsed_gff: &ParsedGff,
    paths: &KeggCatalogPaths,
) -> Result<KeggCatalog, StorageError> {
    if paths.is_empty() {
        return Ok(KeggCatalog::default());
    }
    build_kegg_catalog(parsed_gff, &paths.as_input())
}

fn build_sequence_models<'a>(
    config: &GenomeSnapshotBuild,
    sequences: impl Iterator<Item = &'a crate::fasta::FastaSequence>,
) -> Result<Vec<Sequence>, StorageError> {
    sequences
        .map(|sequence| {
            Ok(Sequence {
                name: SequenceName::new(sequence.name.clone())?,
                assembly_accession: config.assembly.accession.clone(),
                length: sequence.bases.len() as u64,
                refget_checksum: refget_checksum(sequence.bases.as_bytes()),
            })
        })
        .collect()
}

fn assembly_with_refget_checksum(assembly: &Assembly, sequences: &[Sequence]) -> Assembly {
    let mut assembly = assembly.clone();
    assembly.refget_checksum = Some(assembly_checksum(sequences));
    assembly
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
    if let Some(path) = &config.protein_fasta_path {
        apply_protein_sequences(parsed, path)?;
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
                kegg_files: None,
                protein_fasta_file: None,
                orthogroup_file: None,
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
                cdss: Vec::new(),
                kegg_catalog: KeggCatalog::default(),
                orthogroup_catalog: OrthogroupCatalog::default(),
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

    #[test]
    fn read_snapshot_defaults_missing_orthogroup_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.json");
        let mut value = serde_json::to_value(sample_snapshot()).unwrap();
        value
            .get_mut("dataset")
            .and_then(serde_json::Value::as_object_mut)
            .unwrap()
            .remove("orthogroup_catalog");
        value
            .get_mut("manifest")
            .and_then(serde_json::Value::as_object_mut)
            .unwrap()
            .remove("orthogroup_file");
        serde_json::to_writer(std::fs::File::create(&path).unwrap(), &value).unwrap();

        let loaded = read_snapshot(&path).unwrap();

        assert!(loaded.dataset.orthogroup_catalog.groups.is_empty());
        assert!(loaded.manifest.orthogroup_file.is_none());
    }
}
