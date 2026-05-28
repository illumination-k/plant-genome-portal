use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::EpigenomeError;

/// Epigenome assay supported in MVP.
///
/// Only ChIP-seq and ATAC-seq are modelled. CUT&RUN / CUT&Tag / DNase-seq are
/// out of MVP scope; add new variants when those assays are imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Assay {
    ChipSeq,
    AtacSeq,
}

impl Assay {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChipSeq => "chip_seq",
            Self::AtacSeq => "atac_seq",
        }
    }
}

impl Display for Assay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Assay {
    type Err = EpigenomeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "chip_seq" | "ChIP-seq" | "chipseq" => Ok(Self::ChipSeq),
            "atac_seq" | "ATAC-seq" | "atacseq" => Ok(Self::AtacSeq),
            other => Err(EpigenomeError::UnknownAssay(other.to_owned())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_uses_snake_case() {
        let json = serde_json::to_string(&Assay::ChipSeq).unwrap();
        assert_eq!(json, "\"chip_seq\"");
        let parsed: Assay = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Assay::ChipSeq);

        let json = serde_json::to_string(&Assay::AtacSeq).unwrap();
        assert_eq!(json, "\"atac_seq\"");
    }

    #[test]
    fn from_str_accepts_canonical_and_aliases() {
        assert_eq!(Assay::from_str("chip_seq").unwrap(), Assay::ChipSeq);
        assert_eq!(Assay::from_str("ChIP-seq").unwrap(), Assay::ChipSeq);
        assert_eq!(Assay::from_str("atac_seq").unwrap(), Assay::AtacSeq);
        assert_eq!(Assay::from_str("ATAC-seq").unwrap(), Assay::AtacSeq);
    }

    #[test]
    fn as_str_returns_canonical_label() {
        assert_eq!(Assay::ChipSeq.as_str(), "chip_seq");
        assert_eq!(Assay::AtacSeq.as_str(), "atac_seq");
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!(Assay::from_str("rna_seq").is_err());
    }
}
