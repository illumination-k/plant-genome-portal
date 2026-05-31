use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Unit in which an expression value is reported.
///
/// The variants cover the common normalized and count-based units used by
/// downstream pipelines (Salmon, Kallisto, featureCounts, DESeq2, edgeR, ...).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionUnit {
    /// Transcripts per million.
    Tpm,
    /// Fragments per kilobase of transcript per million mapped reads.
    Fpkm,
    /// Reads per kilobase of transcript per million mapped reads.
    Rpkm,
    /// Counts per million.
    Cpm,
    /// Raw read counts (integer-valued, but stored as f64 for matrix uniformity).
    RawCount,
    /// Library-size-normalized counts (e.g. DESeq2 normalized counts, TMM).
    NormalizedCount,
}

impl ExpressionUnit {
    /// `true` if the unit represents discrete read counts rather than a
    /// normalized rate.
    pub const fn is_count_based(self) -> bool {
        matches!(self, Self::RawCount | Self::NormalizedCount)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn count_based_classification() {
        assert!(ExpressionUnit::RawCount.is_count_based());
        assert!(ExpressionUnit::NormalizedCount.is_count_based());
        assert!(!ExpressionUnit::Tpm.is_count_based());
        assert!(!ExpressionUnit::Fpkm.is_count_based());
        assert!(!ExpressionUnit::Rpkm.is_count_based());
        assert!(!ExpressionUnit::Cpm.is_count_based());
    }
}
