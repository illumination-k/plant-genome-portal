use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::DomainError;
use crate::ids::KeggEntryId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct KeggPathwayId(String);

impl KeggPathwayId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = canonicalize_pathway_id(value.into());
        if !is_canonical_pathway_id(&value) {
            return Err(DomainError::InvalidKeggPathwayId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for KeggPathwayId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for KeggPathwayId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn canonicalize_pathway_id(value: String) -> String {
    let trimmed = value.strip_prefix("path:").unwrap_or(&value);
    for prefix in ["ko", "ec"] {
        if let Some(digits) = trimmed.strip_prefix(prefix)
            && digits.len() == 5
            && digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return format!("map{digits}");
        }
    }
    trimmed.to_owned()
}

fn is_canonical_pathway_id(value: &str) -> bool {
    let Some(digits) = value.strip_prefix("map") else {
        return false;
    };
    digits.len() == 5 && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct KeggModuleId(String);

impl KeggModuleId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let normalized = strip_prefix_if(value.into(), "md:");
        if !is_fixed_kegg_code(&normalized, b'M') {
            return Err(DomainError::InvalidKeggModuleId(normalized));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for KeggModuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for KeggModuleId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct KeggReactionId(String);

impl KeggReactionId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let normalized = strip_prefix_if(value.into(), "rn:");
        if !is_fixed_kegg_code(&normalized, b'R') {
            return Err(DomainError::InvalidKeggReactionId(normalized));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for KeggReactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for KeggReactionId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn strip_prefix_if(value: String, prefix: &str) -> String {
    value
        .strip_prefix(prefix)
        .map(str::to_owned)
        .unwrap_or(value)
}

fn is_fixed_kegg_code(value: &str, leading: u8) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 6 && bytes[0] == leading && bytes[1..].iter().all(u8::is_ascii_digit)
}

/// Canonicalize a `KeggEntryId` from `FunctionalAnnotation::Kegg` into a bare
/// KO code (`K00001`) suitable for lookup in [`KeggCatalog::ko_links`].
///
/// Returns `None` when the entry is not a KEGG orthology entry.
pub fn ko_entry_id(entry_id: &KeggEntryId) -> Option<KeggEntryId> {
    let raw = entry_id
        .as_str()
        .strip_prefix("ko:")
        .unwrap_or(entry_id.as_str());
    is_fixed_kegg_code(raw, b'K')
        .then(|| KeggEntryId::new(raw))
        .and_then(Result::ok)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggPathway {
    pub id: KeggPathwayId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggModule {
    pub id: KeggModuleId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggReaction {
    pub id: KeggReactionId,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggKoLinks {
    pub ko: KeggEntryId,
    pub pathways: Vec<KeggPathwayId>,
    pub modules: Vec<KeggModuleId>,
    pub reactions: Vec<KeggReactionId>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggCatalog {
    #[serde(default)]
    pub pathways: Vec<KeggPathway>,
    #[serde(default)]
    pub modules: Vec<KeggModule>,
    #[serde(default)]
    pub reactions: Vec<KeggReaction>,
    #[serde(default)]
    pub ko_links: Vec<KeggKoLinks>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[derive(Clone, Copy, Debug)]
    enum KeggKind {
        Pathway,
        Module,
        Reaction,
    }

    macro_rules! assert_accepts_kegg_identifier {
        ($kind:expr, $value:expr, $expected:expr) => {
            match $kind {
                KeggKind::Pathway => {
                    let value = $value;
                    let expected = $expected;
                    let id = KeggPathwayId::new(value).unwrap();
                    assert_eq!(id.as_str(), expected);
                    assert_eq!(id.to_string(), expected);
                    assert_eq!(id.clone().into_string(), expected);
                }
                KeggKind::Module => {
                    let value = $value;
                    let expected = $expected;
                    let id = KeggModuleId::new(value).unwrap();
                    assert_eq!(id.as_str(), expected);
                    assert_eq!(id.to_string(), expected);
                    assert_eq!(id.clone().into_string(), expected);
                }
                KeggKind::Reaction => {
                    let value = $value;
                    let expected = $expected;
                    let id = KeggReactionId::new(value).unwrap();
                    assert_eq!(id.as_str(), expected);
                    assert_eq!(id.to_string(), expected);
                    assert_eq!(id.clone().into_string(), expected);
                }
            }
        };
    }

    macro_rules! assert_rejects_kegg_identifier {
        ($kind:expr, $value:expr) => {
            let is_err = match $kind {
                KeggKind::Pathway => KeggPathwayId::new($value).is_err(),
                KeggKind::Module => KeggModuleId::new($value).is_err(),
                KeggKind::Reaction => KeggReactionId::new($value).is_err(),
            };
            assert!(is_err, "{:?} accepted invalid value {:?}", $kind, $value);
        };
    }

    #[rstest]
    #[case::pathway_map(KeggKind::Pathway, "map00010", "map00010")]
    #[case::pathway_ko(KeggKind::Pathway, "ko00010", "map00010")]
    #[case::pathway_ec(KeggKind::Pathway, "ec00010", "map00010")]
    #[case::pathway_with_prefix(KeggKind::Pathway, "path:map00010", "map00010")]
    #[case::pathway_ko_with_prefix(KeggKind::Pathway, "path:ko00010", "map00010")]
    #[case::module(KeggKind::Module, "M00001", "M00001")]
    #[case::reaction(KeggKind::Reaction, "R00001", "R00001")]
    #[case::reaction_with_prefix(KeggKind::Reaction, "rn:R00001", "R00001")]
    fn kegg_ids_accept(#[case] kind: KeggKind, #[case] value: &str, #[case] expected: &str) {
        assert_accepts_kegg_identifier!(kind, value, expected);
    }

    #[rstest]
    #[case::pathway_unknown_prefix(KeggKind::Pathway, "xy00010")]
    #[case::pathway_short_digits(KeggKind::Pathway, "map0001")]
    #[case::pathway_long_digits(KeggKind::Pathway, "map000010")]
    #[case::pathway_non_digits(KeggKind::Pathway, "mapABCDE")]
    #[case::module_wrong_prefix(KeggKind::Module, "K00001")]
    #[case::module_short_digits(KeggKind::Module, "M0001")]
    #[case::module_long_digits(KeggKind::Module, "M000001")]
    #[case::module_non_digits(KeggKind::Module, "Mabcde")]
    #[case::reaction_wrong_prefix(KeggKind::Reaction, "K00001")]
    #[case::reaction_short_digits(KeggKind::Reaction, "R0001")]
    #[case::reaction_non_digits(KeggKind::Reaction, "Rabcde")]
    fn kegg_ids_reject(#[case] kind: KeggKind, #[case] value: &str) {
        assert_rejects_kegg_identifier!(kind, value);
    }

    #[test]
    fn ko_entry_id_normalizes_and_filters_orthology_ids() {
        let with_prefix = KeggEntryId::new("ko:K00001").unwrap();
        assert_eq!(ko_entry_id(&with_prefix).unwrap().as_str(), "K00001");

        let bare = KeggEntryId::new("K00001").unwrap();
        assert_eq!(ko_entry_id(&bare).unwrap().as_str(), "K00001");

        let pathway = KeggEntryId::new("map00010").unwrap();
        assert!(ko_entry_id(&pathway).is_none());

        let other = KeggEntryId::new("M00001").unwrap();
        assert!(ko_entry_id(&other).is_none());
    }
}
