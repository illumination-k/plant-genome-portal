use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use genome_domain::{HalfOpenRegion, Strand};

/// One peak call (narrowPeak or broadPeak row), normalised to the portal's
/// internal 0-based half-open coordinate system.
///
/// `summit_offset` is `Some` for narrowPeak (column 10, summit offset from
/// `region.start`) and `None` for broadPeak.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Peak {
    pub region: HalfOpenRegion,
    pub name: String,
    /// BED score (column 5), `0..=1000`. Conventionally
    /// `min(int(-10*log10(qvalue)), 1000)` as written by MACS.
    pub score: u16,
    pub strand: Strand,
    /// MACS `signalValue` column — typically fold enrichment over input.
    pub signal_value: f64,
    /// `-log10(pvalue)`. `-1` in the source file means "not reported".
    pub p_value: f64,
    /// `-log10(qvalue)`. `-1` in the source file means "not reported".
    pub q_value: f64,
    /// Summit offset relative to `region.start`, 0-based. `Some` only for
    /// narrowPeak.
    pub summit_offset: Option<u32>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use genome_domain::{Position0, SequenceName};

    fn region() -> HalfOpenRegion {
        HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(100),
            Position0::new(300),
        )
        .unwrap()
    }

    #[test]
    fn narrow_peak_has_summit() {
        let peak = Peak {
            region: region(),
            name: "peak_1".to_owned(),
            score: 500,
            strand: Strand::Unknown,
            signal_value: 12.3,
            p_value: 30.0,
            q_value: 25.0,
            summit_offset: Some(75),
        };
        let json = serde_json::to_string(&peak).unwrap();
        let parsed: Peak = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, peak);
        assert_eq!(parsed.summit_offset, Some(75));
    }

    #[test]
    fn broad_peak_has_no_summit() {
        let peak = Peak {
            region: region(),
            name: "peak_broad".to_owned(),
            score: 200,
            strand: Strand::Unknown,
            signal_value: 4.0,
            p_value: -1.0,
            q_value: 5.0,
            summit_offset: None,
        };
        assert!(peak.summit_offset.is_none());
    }
}
