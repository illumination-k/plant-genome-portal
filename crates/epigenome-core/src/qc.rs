use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Quality-control metrics for a ChIP-seq / ATAC-seq experiment, following
/// ENCODE conventions.
///
/// All fields are `Option` because public datasets often only report a subset.
/// Pass thresholds (for surfacing badges in the UI):
/// * FRiP ≥ 0.01 (ChIP) / ≥ 0.20 (ATAC)
/// * NSC ≥ 1.05, RSC ≥ 0.8 (ChIP cross-correlation)
/// * NRF ≥ 0.8 (library complexity)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ExperimentQc {
    /// Fraction of Reads in Peaks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frip: Option<f64>,
    /// Non-Redundant read Fraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nrf: Option<f64>,
    /// Normalized Strand-cross-correlation Coefficient.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsc: Option<f64>,
    /// Relative Strand-cross-correlation Coefficient.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rsc: Option<f64>,
    /// Number of mapped reads in the alignment used for peak calling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapped_reads: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_fields_are_skipped_in_serialization() {
        #[allow(clippy::unwrap_used)]
        let json = serde_json::to_string(&ExperimentQc::default()).unwrap();
        assert_eq!(json, "{}");

        let qc = ExperimentQc {
            frip: Some(0.42),
            ..Default::default()
        };
        #[allow(clippy::unwrap_used)]
        let json = serde_json::to_string(&qc).unwrap();
        assert_eq!(json, "{\"frip\":0.42}");
    }
}
