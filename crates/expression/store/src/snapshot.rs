use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::dataset::ExpressionDataset;
use crate::error::ExpressionStoreError;

/// Provenance for an [`ExpressionSnapshot`]. Mirrors `SnapshotManifest` in the
/// genome-store crate but is intentionally lighter — expression data has
/// no fixed file layout yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpressionSnapshotManifest {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionSnapshot {
    pub manifest: ExpressionSnapshotManifest,
    pub dataset: ExpressionDataset,
}

pub fn read_snapshot(path: impl AsRef<Path>) -> Result<ExpressionSnapshot, ExpressionStoreError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

pub fn write_snapshot(
    path: impl AsRef<Path>,
    snapshot: &ExpressionSnapshot,
) -> Result<(), ExpressionStoreError> {
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, snapshot)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use expression_domain::{ExpressionMatrix, ExpressionUnit, SraRunAccession};
    use genome_domain::{AssemblyAccession, GeneId};

    use super::*;
    use crate::dataset::ExpressionDataset;

    fn sample_snapshot() -> ExpressionSnapshot {
        let assembly = AssemblyAccession::new("GCA_test").unwrap();
        let matrix = ExpressionMatrix::new(
            assembly.clone(),
            ExpressionUnit::Tpm,
            vec![GeneId::new("Mp1g00010").unwrap()],
            vec![SraRunAccession::new("SRR000001").unwrap()],
            vec![1.5],
        )
        .unwrap();
        ExpressionSnapshot {
            manifest: ExpressionSnapshotManifest {
                source: "test".to_owned(),
                description: None,
            },
            dataset: ExpressionDataset {
                assembly_accession: assembly,
                bioprojects: Vec::new(),
                samples: Vec::new(),
                matrices: vec![matrix],
            },
        }
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("expression_snapshot.json");
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
        let snapshot: ExpressionSnapshot = serde_json::from_value(json).unwrap();
        assert!(snapshot.dataset.bioprojects.is_empty());
        assert!(snapshot.dataset.samples.is_empty());
        assert!(snapshot.dataset.matrices.is_empty());
    }
}
