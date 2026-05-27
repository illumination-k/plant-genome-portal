use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Whether a peak file is narrowPeak (BED6+4, has summit offset) or broadPeak
/// (BED6+3, no summit).
///
/// Histone marks form broad domains (H3K27me3, H3K9me3, H3K36me3) and are
/// usually called with `--broad`; sharp marks (H3K4me3, H3K27ac) and
/// transcription factors give narrow peaks. ATAC-seq peaks are narrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PeakKind {
    Narrow,
    Broad,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_uses_snake_case() {
        #[allow(clippy::unwrap_used)]
        let json = serde_json::to_string(&PeakKind::Narrow).unwrap();
        assert_eq!(json, "\"narrow\"");
        #[allow(clippy::unwrap_used)]
        let parsed: PeakKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PeakKind::Narrow);
    }
}
