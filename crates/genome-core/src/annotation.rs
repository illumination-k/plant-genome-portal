use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use crate::ids::{GoTermId, InterProId, KeggEntryId, KogEntryId, NcbiFamAccession, PfamAccession};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationSource {
    InterProScan,
    Go,
    Kegg,
    Manual,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AnnotationEvidence {
    pub source: AnnotationSource,
    pub method: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

impl AnnotationEvidence {
    pub fn new(source: AnnotationSource) -> Self {
        Self {
            source,
            method: None,
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FunctionalAnnotation {
    InterPro(InterProAnnotation),
    Pfam(PfamAnnotation),
    NcbiFam(NcbiFamAnnotation),
    Kog(KogAnnotation),
    GoTerm(GoTermAnnotation),
    Kegg(KeggAnnotation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct InterProAnnotation {
    pub interpro_id: InterProId,
    pub name: Option<String>,
    pub evidence: AnnotationEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PfamAnnotation {
    pub accession: PfamAccession,
    pub name: Option<String>,
    pub interpro_id: Option<InterProId>,
    pub evidence: AnnotationEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NcbiFamAnnotation {
    pub accession: NcbiFamAccession,
    pub name: Option<String>,
    pub interpro_id: Option<InterProId>,
    pub evidence: AnnotationEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KogAnnotation {
    pub entry_id: KogEntryId,
    pub name: Option<String>,
    pub interpro_id: Option<InterProId>,
    pub evidence: AnnotationEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GoTermAnnotation {
    pub term_id: GoTermId,
    pub name: Option<String>,
    pub namespace: Option<GoNamespace>,
    pub evidence: AnnotationEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoNamespace {
    BiologicalProcess,
    MolecularFunction,
    CellularComponent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct KeggAnnotation {
    pub entry_id: KeggEntryId,
    pub entry_kind: KeggEntryKind,
    pub name: Option<String>,
    pub evidence: AnnotationEvidence,
}

impl KeggAnnotation {
    pub fn new(entry_id: KeggEntryId, name: Option<String>, evidence: AnnotationEvidence) -> Self {
        let entry_kind = KeggEntryKind::from_entry_id(entry_id.as_str());
        Self {
            entry_id,
            entry_kind,
            name,
            evidence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KeggEntryKind {
    Orthology,
    Pathway,
    Module,
    Reaction,
    Compound,
    Glycan,
    Other,
}

impl KeggEntryKind {
    pub fn from_entry_id(entry_id: &str) -> Self {
        let normalized = entry_id
            .strip_prefix("ko:")
            .or_else(|| entry_id.strip_prefix("path:"))
            .unwrap_or(entry_id);

        if normalized.len() == 6
            && normalized.starts_with('K')
            && normalized[1..].bytes().all(|byte| byte.is_ascii_digit())
        {
            Self::Orthology
        } else if normalized.starts_with("map")
            || normalized.starts_with("ko")
            || normalized.starts_with("ec")
        {
            Self::Pathway
        } else if normalized.len() == 6
            && normalized.starts_with('M')
            && normalized[1..].bytes().all(|byte| byte.is_ascii_digit())
        {
            Self::Module
        } else if normalized.len() == 6
            && normalized.starts_with('R')
            && normalized[1..].bytes().all(|byte| byte.is_ascii_digit())
        {
            Self::Reaction
        } else if normalized.len() == 6
            && normalized.starts_with('C')
            && normalized[1..].bytes().all(|byte| byte.is_ascii_digit())
        {
            Self::Compound
        } else if normalized.len() == 6
            && normalized.starts_with('G')
            && normalized[1..].bytes().all(|byte| byte.is_ascii_digit())
        {
            Self::Glycan
        } else {
            Self::Other
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn kegg_entry_kind_classifies_each_branch() {
        let cases = [
            ("K00001", KeggEntryKind::Orthology),
            ("ko:K12345", KeggEntryKind::Orthology),
            ("path:map00010", KeggEntryKind::Pathway),
            ("map00010", KeggEntryKind::Pathway),
            ("ko00010", KeggEntryKind::Pathway),
            ("ec00010", KeggEntryKind::Pathway),
            ("M00001", KeggEntryKind::Module),
            ("R00001", KeggEntryKind::Reaction),
            ("C00001", KeggEntryKind::Compound),
            ("G00001", KeggEntryKind::Glycan),
        ];
        for (input, expected) in cases {
            assert_eq!(
                KeggEntryKind::from_entry_id(input),
                expected,
                "input: {input}"
            );
        }
    }

    #[test]
    fn kegg_entry_kind_falls_back_to_other() {
        // Wrong prefix letter — none of the 6-char single-letter branches match.
        assert_eq!(KeggEntryKind::from_entry_id("X00001"), KeggEntryKind::Other);
        // Right prefix letter but wrong length — length check must reject.
        assert_eq!(KeggEntryKind::from_entry_id("K0001"), KeggEntryKind::Other);
        assert_eq!(
            KeggEntryKind::from_entry_id("M0000001"),
            KeggEntryKind::Other
        );
        assert_eq!(KeggEntryKind::from_entry_id("R0001"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("C0001"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("G0001"), KeggEntryKind::Other);
        // Right prefix letter and length but non-digit tail — digit check must reject.
        assert_eq!(KeggEntryKind::from_entry_id("Kabcde"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("Mabcde"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("Rabcde"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("Cabcde"), KeggEntryKind::Other);
        assert_eq!(KeggEntryKind::from_entry_id("Gabcde"), KeggEntryKind::Other);
        // Pathway prefix arms must each be required: dropping one prefix string would
        // still hit the others, so verify "ec" alone classifies even with no map/ko.
        assert_eq!(
            KeggEntryKind::from_entry_id("ec99999"),
            KeggEntryKind::Pathway
        );
    }
}
