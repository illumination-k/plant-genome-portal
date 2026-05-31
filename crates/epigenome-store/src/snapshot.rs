use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::dataset::EpigenomeDataset;
use crate::error::EpigenomeStoreError;

/// Provenance for an [`EpigenomeSnapshot`]. Mirrors `ExpressionSnapshotManifest`
/// — epigenome data has no fixed file layout yet (curator-driven).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpigenomeSnapshotManifest {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpigenomeSnapshot {
    pub manifest: EpigenomeSnapshotManifest,
    pub dataset: EpigenomeDataset,
}

pub fn read_snapshot(path: impl AsRef<Path>) -> Result<EpigenomeSnapshot, EpigenomeStoreError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn write_snapshot(
    path: impl AsRef<Path>,
    snapshot: &EpigenomeSnapshot,
) -> Result<(), EpigenomeStoreError> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, snapshot)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use epigenome_core::{Assay, ExperimentId, PeakKind};
    use genome_core::AssemblyAccession;

    use crate::dataset::{EpigenomeDataset, ExperimentPeaks};

    fn sample_snapshot() -> EpigenomeSnapshot {
        let assembly = AssemblyAccession::new("GCA_test").unwrap();
        EpigenomeSnapshot {
            manifest: EpigenomeSnapshotManifest {
                source: "test".to_owned(),
                description: None,
            },
            dataset: EpigenomeDataset {
                assembly_accession: assembly.clone(),
                experiments: vec![epigenome_core::Experiment {
                    id: ExperimentId::new("exp1").unwrap(),
                    assay: Assay::AtacSeq,
                    target: None,
                    antibody: None,
                    assembly_accession: assembly.clone(),
                    geo_series: None,
                    geo_sample: None,
                    sra_runs: Vec::new(),
                    tissue: None,
                    dev_stage: None,
                    treatment: None,
                    replicate: None,
                    pipeline: None,
                    qvalue_cutoff: None,
                    qc: epigenome_core::ExperimentQc::default(),
                    peak_kind: PeakKind::Narrow,
                    signal_file: None,
                    attributes: std::collections::BTreeMap::new(),
                }],
                peaks: vec![ExperimentPeaks {
                    experiment_id: ExperimentId::new("exp1").unwrap(),
                    kind: PeakKind::Narrow,
                    peaks: Vec::new(),
                }],
            },
        }
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("epigenome_snapshot.json");
        let snapshot = sample_snapshot();

        write_snapshot(&path, &snapshot).unwrap();
        let loaded = read_snapshot(&path).unwrap();
        assert_eq!(loaded, snapshot);
    }

    #[test]
    fn dataset_fields_default_to_empty_when_omitted() {
        let json = serde_json::json!({
            "manifest": { "source": "test" },
            "dataset": { "assembly_accession": "GCA_test" }
        });
        let snapshot: EpigenomeSnapshot = serde_json::from_value(json).unwrap();
        assert!(snapshot.dataset.experiments.is_empty());
        assert!(snapshot.dataset.peaks.is_empty());
    }
}
