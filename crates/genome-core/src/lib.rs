//! Core genome domain types.
//!
//! This crate intentionally has no I/O or async dependencies.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("{kind} must not be empty")]
    EmptyIdentifier { kind: &'static str },
    #[error("invalid GO term id: {0}")]
    InvalidGoTermId(String),
    #[error("invalid InterPro id: {0}")]
    InvalidInterProId(String),
    #[error("invalid Pfam accession: {0}")]
    InvalidPfamAccession(String),
    #[error("1-based positions must be greater than zero")]
    ZeroPosition1,
    #[error("region start must be <= end")]
    InvalidClosedRegion,
    #[error("region start must be < end")]
    InvalidHalfOpenRegion,
    #[error("invalid strand: {0}")]
    InvalidStrand(String),
    #[error("invalid region expression: {0}")]
    InvalidRegionExpression(String),
}

macro_rules! string_id {
    ($name:ident, $kind:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(DomainError::EmptyIdentifier { kind: $kind });
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

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = DomainError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

string_id!(AssemblyAccession, "assembly accession");
string_id!(GeneId, "gene id");
string_id!(KeggEntryId, "KEGG entry id");
string_id!(KogEntryId, "KOG entry id");
string_id!(NcbiFamAccession, "NCBIfam accession");
string_id!(SequenceName, "sequence name");
string_id!(TranscriptId, "transcript id");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct GoTermId(String);

impl GoTermId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if !is_go_term_id(&value) {
            return Err(DomainError::InvalidGoTermId(value));
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

impl Display for GoTermId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for GoTermId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn is_go_term_id(value: &str) -> bool {
    let Some(digits) = value.strip_prefix("GO:") else {
        return false;
    };
    digits.len() == 7 && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct InterProId(String);

impl InterProId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if !is_interpro_id(&value) {
            return Err(DomainError::InvalidInterProId(value));
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

impl Display for InterProId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for InterProId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn is_interpro_id(value: &str) -> bool {
    let Some(digits) = value.strip_prefix("IPR") else {
        return false;
    };
    digits.len() == 6 && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
pub struct PfamAccession(String);

impl PfamAccession {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if !is_pfam_accession(&value) {
            return Err(DomainError::InvalidPfamAccession(value));
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

impl Display for PfamAccession {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PfamAccession {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

fn is_pfam_accession(value: &str) -> bool {
    let Some(digits) = value.strip_prefix("PF") else {
        return false;
    };
    digits.len() == 5 && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct TaxId(u32);

impl TaxId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Display for TaxId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct Position0(u64);

impl Position0 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct Position1(u64);

impl Position1 {
    pub fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::ZeroPosition1);
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn to_position0(self) -> Position0 {
        Position0(self.0 - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClosedRegion {
    pub sequence_name: SequenceName,
    pub start: Position1,
    pub end: Position1,
}

impl ClosedRegion {
    pub fn new(
        sequence_name: SequenceName,
        start: Position1,
        end: Position1,
    ) -> Result<Self, DomainError> {
        if start.get() > end.get() {
            return Err(DomainError::InvalidClosedRegion);
        }
        Ok(Self {
            sequence_name,
            start,
            end,
        })
    }

    pub fn to_half_open(&self) -> Result<HalfOpenRegion, DomainError> {
        HalfOpenRegion::new(
            self.sequence_name.clone(),
            self.start.to_position0(),
            Position0::new(self.end.get()),
        )
    }
}

impl FromStr for ClosedRegion {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((sequence_name, range)) = value.split_once(':') else {
            return Err(DomainError::InvalidRegionExpression(value.to_owned()));
        };
        let Some((start, end)) = range.split_once('-') else {
            return Err(DomainError::InvalidRegionExpression(value.to_owned()));
        };

        let start = start
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| DomainError::InvalidRegionExpression(value.to_owned()))?;
        let end = end
            .replace(',', "")
            .parse::<u64>()
            .map_err(|_| DomainError::InvalidRegionExpression(value.to_owned()))?;

        Self::new(
            SequenceName::new(sequence_name)?,
            Position1::new(start)?,
            Position1::new(end)?,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct HalfOpenRegion {
    pub sequence_name: SequenceName,
    pub start: Position0,
    pub end: Position0,
}

impl HalfOpenRegion {
    pub fn new(
        sequence_name: SequenceName,
        start: Position0,
        end: Position0,
    ) -> Result<Self, DomainError> {
        if start.get() >= end.get() {
            return Err(DomainError::InvalidHalfOpenRegion);
        }
        Ok(Self {
            sequence_name,
            start,
            end,
        })
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.sequence_name == other.sequence_name
            && self.start.get() < other.end.get()
            && other.start.get() < self.end.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strand {
    Forward,
    Reverse,
    Unknown,
}

impl FromStr for Strand {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "+" => Ok(Self::Forward),
            "-" => Ok(Self::Reverse),
            "." | "?" => Ok(Self::Unknown),
            other => Err(DomainError::InvalidStrand(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssemblySource {
    Ncbi,
    MarpolBase,
    Tair,
    Phytozome,
    Community,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Taxon {
    pub tax_id: TaxId,
    pub scientific_name: String,
    pub common_name: Option<String>,
    pub rank: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Assembly {
    pub accession: AssemblyAccession,
    pub tax_id: TaxId,
    pub name: String,
    pub source: AssemblySource,
    pub refget_checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Sequence {
    pub name: SequenceName,
    pub assembly_accession: AssemblyAccession,
    pub length: u64,
    pub refget_checksum: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Gene {
    pub id: GeneId,
    pub assembly_accession: AssemblyAccession,
    pub symbol: Option<String>,
    pub locus_tag: Option<String>,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
    pub feature_type: String,
    pub annotations: Vec<FunctionalAnnotation>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Transcript {
    pub id: TranscriptId,
    pub gene_id: GeneId,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
    pub feature_type: String,
    pub annotations: Vec<FunctionalAnnotation>,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Exon {
    pub transcript_id: TranscriptId,
    pub sequence_name: SequenceName,
    pub region: HalfOpenRegion,
    pub strand: Strand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GeneRecord {
    pub gene: Gene,
    pub transcripts: Vec<Transcript>,
    pub exons: Vec<Exon>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GenomeDataset {
    pub taxon: Taxon,
    pub assembly: Assembly,
    pub sequences: Vec<Sequence>,
    pub genes: Vec<Gene>,
    pub transcripts: Vec<Transcript>,
    pub exons: Vec<Exon>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneSearch {
    pub tax_id: Option<TaxId>,
    pub symbol: Option<String>,
    pub locus_tag: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

pub trait GenomeRepository: Send + Sync + 'static {
    fn taxon(&self, tax_id: TaxId) -> Option<Taxon>;
    fn assembly(&self, accession: &AssemblyAccession) -> Option<Assembly>;
    fn assemblies_for_taxon(&self, tax_id: TaxId) -> Vec<Assembly>;
    fn sequences_for_assembly(&self, accession: &AssemblyAccession) -> Vec<Sequence>;
    fn sequence_by_checksum(&self, checksum: &str) -> Option<Sequence>;
    fn gene(&self, gene_id: &GeneId) -> Option<GeneRecord>;
    fn search_genes(&self, search: &GeneSearch) -> Vec<Gene>;
    fn features_in_region(
        &self,
        accession: &AssemblyAccession,
        region: &HalfOpenRegion,
    ) -> Vec<Gene>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn closed_region_converts_to_half_open() {
        let region = ClosedRegion::from_str("chr1:1-10").unwrap();
        let half_open = region.to_half_open().unwrap();

        assert_eq!(half_open.start.get(), 0);
        assert_eq!(half_open.end.get(), 10);
    }

    #[test]
    fn half_open_overlap_uses_sequence_name() {
        let a = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(0),
            Position0::new(10),
        )
        .unwrap();
        let b = HalfOpenRegion::new(
            SequenceName::new("chr1").unwrap(),
            Position0::new(9),
            Position0::new(20),
        )
        .unwrap();
        let c = HalfOpenRegion::new(
            SequenceName::new("chr2").unwrap(),
            Position0::new(9),
            Position0::new(20),
        )
        .unwrap();

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn go_term_id_requires_canonical_accession() {
        assert_eq!(GoTermId::new("GO:0008150").unwrap().as_str(), "GO:0008150");
        assert!(GoTermId::new("0008150").is_err());
        assert!(GoTermId::new("GO:8150").is_err());
    }

    #[test]
    fn interpro_id_requires_canonical_identifier() {
        assert_eq!(InterProId::new("IPR000001").unwrap().as_str(), "IPR000001");
        assert!(InterProId::new("PF00001").is_err());
        assert_eq!(PfamAccession::new("PF00001").unwrap().as_str(), "PF00001");
    }

    #[test]
    fn kegg_entry_kind_infers_common_entry_types() {
        assert_eq!(
            KeggEntryKind::from_entry_id("K00001"),
            KeggEntryKind::Orthology
        );
        assert_eq!(
            KeggEntryKind::from_entry_id("path:map00010"),
            KeggEntryKind::Pathway
        );
        assert_eq!(
            KeggEntryKind::from_entry_id("M00001"),
            KeggEntryKind::Module
        );
    }
}
