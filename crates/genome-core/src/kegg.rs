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

    #[test]
    fn pathway_id_canonicalizes_ko_and_ec_prefixes_to_map() {
        assert_eq!(KeggPathwayId::new("map00010").unwrap().as_str(), "map00010");
        assert_eq!(KeggPathwayId::new("ko00010").unwrap().as_str(), "map00010");
        assert_eq!(KeggPathwayId::new("ec00010").unwrap().as_str(), "map00010");
        assert_eq!(
            KeggPathwayId::new("path:map00010").unwrap().as_str(),
            "map00010"
        );
        assert_eq!(
            KeggPathwayId::new("path:ko00010").unwrap().as_str(),
            "map00010"
        );
    }

    #[test]
    fn pathway_id_rejects_unknown_prefix_and_wrong_digit_count() {
        assert!(KeggPathwayId::new("xy00010").is_err());
        assert!(KeggPathwayId::new("map0001").is_err());
        assert!(KeggPathwayId::new("map000010").is_err());
        assert!(KeggPathwayId::new("mapABCDE").is_err());
    }

    #[test]
    fn module_id_requires_six_chars_starting_with_m() {
        assert_eq!(KeggModuleId::new("M00001").unwrap().as_str(), "M00001");
        assert!(KeggModuleId::new("K00001").is_err());
        assert!(KeggModuleId::new("M0001").is_err());
        assert!(KeggModuleId::new("M000001").is_err());
        assert!(KeggModuleId::new("Mabcde").is_err());
    }

    #[test]
    fn reaction_id_requires_six_chars_starting_with_r() {
        assert_eq!(KeggReactionId::new("R00001").unwrap().as_str(), "R00001");
        assert_eq!(KeggReactionId::new("rn:R00001").unwrap().as_str(), "R00001");
        assert!(KeggReactionId::new("K00001").is_err());
        assert!(KeggReactionId::new("R0001").is_err());
        assert!(KeggReactionId::new("Rabcde").is_err());
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
