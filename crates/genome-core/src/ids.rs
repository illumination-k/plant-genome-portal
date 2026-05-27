use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::DomainError;

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
string_id!(OrthogroupId, "orthogroup id");
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[derive(Clone, Copy, Debug)]
    enum ControlledKind {
        GoTerm,
        InterPro,
        Pfam,
    }

    macro_rules! assert_accepts_identifier {
        ($kind:expr, $value:expr) => {
            match $kind {
                ControlledKind::GoTerm => {
                    let value = $value;
                    let id = GoTermId::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                ControlledKind::InterPro => {
                    let value = $value;
                    let id = InterProId::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                ControlledKind::Pfam => {
                    let value = $value;
                    let id = PfamAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
            }
        };
    }

    macro_rules! assert_rejects_identifier {
        ($kind:expr, $value:expr) => {
            let is_err = match $kind {
                ControlledKind::GoTerm => GoTermId::new($value).is_err(),
                ControlledKind::InterPro => InterProId::new($value).is_err(),
                ControlledKind::Pfam => PfamAccession::new($value).is_err(),
            };
            assert!(is_err, "{:?} accepted invalid value {:?}", $kind, $value);
        };
    }

    #[rstest]
    #[case::go_term(ControlledKind::GoTerm, "GO:0008150")]
    #[case::interpro(ControlledKind::InterPro, "IPR000001")]
    #[case::pfam(ControlledKind::Pfam, "PF00001")]
    fn controlled_ids_accept(#[case] kind: ControlledKind, #[case] value: &str) {
        assert_accepts_identifier!(kind, value);
    }

    #[rstest]
    #[case::go_term_missing_prefix(ControlledKind::GoTerm, "0008150")]
    #[case::go_term_short_digits(ControlledKind::GoTerm, "GO:8150")]
    #[case::go_term_non_digits(ControlledKind::GoTerm, "GO:abcdefg")]
    #[case::interpro_wrong_prefix(ControlledKind::InterPro, "PF00001")]
    #[case::interpro_non_digits(ControlledKind::InterPro, "IPRabcdef")]
    #[case::interpro_wrong_length(ControlledKind::InterPro, "IPR0000001")]
    #[case::pfam_wrong_prefix(ControlledKind::Pfam, "XX00001")]
    #[case::pfam_non_digits(ControlledKind::Pfam, "PFabcde")]
    #[case::pfam_wrong_length(ControlledKind::Pfam, "PF000001")]
    fn controlled_ids_reject(#[case] kind: ControlledKind, #[case] value: &str) {
        assert_rejects_identifier!(kind, value);
    }

    #[test]
    fn tax_id_exposes_inner_value() {
        let tax = TaxId::new(3197);
        assert_eq!(tax.get(), 3197);
        assert_eq!(tax.to_string(), "3197");
    }
}
