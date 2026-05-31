//! Curator-facing TOML manifest reader.
//!
//! The manifest is the single source of truth for which experiments belong
//! to a snapshot, where their peak / signal files live, and what their
//! biological + QC metadata are. `portal-cli import epigenome-manifest`
//! reads it, parses each referenced peak file, and writes
//! `epigenome_snapshot.json`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use epigenome_domain::{
    Antibody, Assay, ExperimentId, GeoSampleAccession, GeoSeriesAccession, PeakKind, Target,
};
use expression_domain::SraRunAccession;
use genome_domain::AssemblyAccession;
use serde::Deserialize;

use crate::error::EpigenomeStoreError;

#[derive(Debug, Clone, Deserialize)]
struct ManifestFile {
    #[serde(default)]
    experiment: Vec<ExperimentManifestEntry>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct ManifestQc {
    #[serde(default)]
    pub frip: Option<f64>,
    #[serde(default)]
    pub nrf: Option<f64>,
    #[serde(default)]
    pub nsc: Option<f64>,
    #[serde(default)]
    pub rsc: Option<f64>,
    #[serde(default)]
    pub mapped_reads: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ExperimentManifestEntry {
    pub id: ExperimentId,
    pub assay: Assay,
    #[serde(default)]
    pub target: Option<Target>,
    #[serde(default)]
    pub antibody: Option<Antibody>,
    pub assembly_accession: AssemblyAccession,
    #[serde(default)]
    pub geo_series: Option<GeoSeriesAccession>,
    #[serde(default)]
    pub geo_sample: Option<GeoSampleAccession>,
    #[serde(default)]
    pub sra_runs: Vec<SraRunAccession>,
    #[serde(default)]
    pub tissue: Option<String>,
    #[serde(default)]
    pub dev_stage: Option<String>,
    #[serde(default)]
    pub treatment: Option<String>,
    #[serde(default)]
    pub replicate: Option<u16>,
    #[serde(default)]
    pub pipeline: Option<String>,
    #[serde(default)]
    pub qvalue_cutoff: Option<f64>,
    #[serde(default)]
    pub qc: ManifestQc,
    pub peak_kind: PeakKind,
    /// Path to the narrowPeak / broadPeak file, relative to the manifest.
    pub peak_file: PathBuf,
    /// Path to the bigWig signal file, relative to the manifest. Optional.
    #[serde(default)]
    pub signal_file: Option<PathBuf>,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

impl ExperimentManifestEntry {
    /// Resolve `peak_file` against the manifest's directory.
    pub fn peak_path(&self, manifest_dir: &Path) -> PathBuf {
        manifest_dir.join(&self.peak_file)
    }

    /// Resolve `signal_file` against the manifest's directory, if set.
    pub fn signal_path(&self, manifest_dir: &Path) -> Option<PathBuf> {
        self.signal_file
            .as_ref()
            .map(|path| manifest_dir.join(path))
    }
}

pub fn parse_manifest(
    path: impl AsRef<Path>,
) -> Result<Vec<ExperimentManifestEntry>, EpigenomeStoreError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)?;
    let file: ManifestFile = toml::from_str(&contents)?;
    if file.experiment.is_empty() {
        return Err(EpigenomeStoreError::InvalidManifest(
            "manifest contains no [[experiment]] entries".to_owned(),
        ));
    }
    Ok(file.experiment)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[[experiment]]
id = "mp_h3k4me3_thallus_rep1"
assay = "chip_seq"
target = "H3K4me3"
antibody = "ab8580"
assembly_accession = "GCA_037833805.1"
geo_series = "GSE123456"
geo_sample = "GSM7890123"
sra_runs = ["SRR12345678"]
tissue = "thallus"
replicate = 1
pipeline = "MACS2 2.2.9 q<0.01"
qvalue_cutoff = 0.01
peak_kind = "narrow"
peak_file = "peaks/mp_h3k4me3_thallus_rep1.narrowPeak.gz"
signal_file = "signal/mp_h3k4me3_thallus_rep1.bw"

[experiment.qc]
frip = 0.42
nsc = 1.8

[experiment.attributes]
sample_label = "thallus H3K4me3 rep1"

[[experiment]]
id = "mp_atac_thallus_rep1"
assay = "atac_seq"
assembly_accession = "GCA_037833805.1"
peak_kind = "narrow"
peak_file = "peaks/mp_atac_thallus_rep1.narrowPeak.gz"
"#;

    #[test]
    fn parses_full_and_minimal_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("epigenome.toml");
        std::fs::write(&path, SAMPLE).unwrap();

        let entries = parse_manifest(&path).unwrap();
        assert_eq!(entries.len(), 2);

        let chip = &entries[0];
        assert_eq!(chip.id.as_str(), "mp_h3k4me3_thallus_rep1");
        assert_eq!(chip.assay, Assay::ChipSeq);
        assert_eq!(chip.target.as_ref().unwrap().as_str(), "H3K4me3");
        assert_eq!(chip.qc.frip, Some(0.42));
        assert_eq!(chip.qc.nsc, Some(1.8));
        assert_eq!(chip.qc.rsc, None);
        assert_eq!(
            chip.attributes.get("sample_label").map(String::as_str),
            Some("thallus H3K4me3 rep1")
        );
        assert!(chip.signal_file.is_some());

        let atac = &entries[1];
        assert_eq!(atac.assay, Assay::AtacSeq);
        assert!(atac.target.is_none());
        assert!(atac.signal_file.is_none());
        assert_eq!(atac.qc, ManifestQc::default());
    }

    #[test]
    fn empty_manifest_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.toml");
        std::fs::write(&path, "").unwrap();
        let err = parse_manifest(&path).unwrap_err();
        assert!(matches!(err, EpigenomeStoreError::InvalidManifest(_)));
    }

    #[test]
    fn resolves_paths_against_manifest_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("epigenome.toml");
        std::fs::write(&path, SAMPLE).unwrap();

        let entries = parse_manifest(&path).unwrap();
        let manifest_dir = path.parent().unwrap();
        let peak_path = entries[0].peak_path(manifest_dir);
        assert!(peak_path.ends_with("peaks/mp_h3k4me3_thallus_rep1.narrowPeak.gz"));
        assert!(entries[0].signal_path(manifest_dir).is_some());
        assert!(entries[1].signal_path(manifest_dir).is_none());
    }
}
