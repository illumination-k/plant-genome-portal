use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use expression_domain::SraRunAccession;
use genome_domain::AssemblyAccession;

use crate::assay::Assay;
use crate::ids::{Antibody, ExperimentId, GeoSampleAccession, GeoSeriesAccession, Target};
use crate::peak_kind::PeakKind;
use crate::qc::ExperimentQc;

/// One ChIP-seq or ATAC-seq experiment.
///
/// "Experiment" here means *one peak-called dataset* — usually one biological
/// replicate that has been aligned and processed end-to-end. Replicates are
/// kept as separate `Experiment`s rather than merged; this mirrors how
/// ChIP-Atlas / ENCODE expose data and lets users see per-replicate QC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Experiment {
    pub id: ExperimentId,
    pub assay: Assay,
    /// Histone mark (`H3K4me3`, `H3K27ac`, …) or TF symbol for ChIP-seq.
    /// `None` for ATAC-seq.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Target>,
    /// Antibody identifier (vendor catalog number, lot, etc.). `None` for
    /// ATAC-seq and for ChIP-seq where the antibody was not reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub antibody: Option<Antibody>,
    pub assembly_accession: AssemblyAccession,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geo_series: Option<GeoSeriesAccession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geo_sample: Option<GeoSampleAccession>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sra_runs: Vec<SraRunAccession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tissue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replicate: Option<u16>,
    /// Free-form pipeline description, e.g. `"MACS2 2.2.9 q<0.01"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qvalue_cutoff: Option<f64>,
    #[serde(default)]
    pub qc: ExperimentQc,
    pub peak_kind: PeakKind,
    /// Basename of the bigWig signal file (resolved against
    /// `--epigenome-signal-root` by the API at config time). `None` when no
    /// signal track is available for this experiment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal_file: Option<String>,
    /// Free-form extension point for curator-specific keys (e.g. raw
    /// repository fields that don't have a first-class home yet).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn minimal_experiment() -> Experiment {
        Experiment {
            id: ExperimentId::new("mp_atac_thallus_rep1").unwrap(),
            assay: Assay::AtacSeq,
            target: None,
            antibody: None,
            assembly_accession: AssemblyAccession::new("GCA_037833805.1").unwrap(),
            geo_series: None,
            geo_sample: None,
            sra_runs: Vec::new(),
            tissue: Some("thallus".to_owned()),
            dev_stage: None,
            treatment: None,
            replicate: Some(1),
            pipeline: None,
            qvalue_cutoff: None,
            qc: ExperimentQc::default(),
            peak_kind: PeakKind::Narrow,
            signal_file: None,
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn experiment_roundtrips_through_json() {
        let experiment = minimal_experiment();
        let json = serde_json::to_string(&experiment).unwrap();
        let parsed: Experiment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, experiment);
    }

    #[test]
    fn missing_optional_fields_default_to_none_or_empty() {
        let json = serde_json::json!({
            "id": "exp_min",
            "assay": "chip_seq",
            "assembly_accession": "GCA_test",
            "peak_kind": "narrow"
        });
        let parsed: Experiment = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.target, None);
        assert_eq!(parsed.qc, ExperimentQc::default());
        assert!(parsed.attributes.is_empty());
        assert!(parsed.sra_runs.is_empty());
    }
}
