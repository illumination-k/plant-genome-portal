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

    #[test]
    fn go_term_id_requires_canonical_accession() {
        let id = GoTermId::new("GO:0008150").unwrap();
        assert_eq!(id.as_str(), "GO:0008150");
        assert_eq!(id.to_string(), "GO:0008150");
        assert_eq!(id.clone().into_string(), "GO:0008150");
        assert!(GoTermId::new("0008150").is_err());
        assert!(GoTermId::new("GO:8150").is_err());
        assert!(GoTermId::new("GO:abcdefg").is_err());
    }

    #[test]
    fn interpro_id_requires_canonical_identifier() {
        let id = InterProId::new("IPR000001").unwrap();
        assert_eq!(id.as_str(), "IPR000001");
        assert_eq!(id.to_string(), "IPR000001");
        assert_eq!(id.clone().into_string(), "IPR000001");
        assert!(InterProId::new("PF00001").is_err());
        // Right prefix, wrong digit-only-tail: catches && -> || in is_interpro_id.
        assert!(InterProId::new("IPRabcdef").is_err());
        // Right prefix, wrong length: catches len-check mutations.
        assert!(InterProId::new("IPR0000001").is_err());
    }

    #[test]
    fn pfam_accession_requires_canonical_identifier() {
        let acc = PfamAccession::new("PF00001").unwrap();
        assert_eq!(acc.as_str(), "PF00001");
        assert_eq!(acc.to_string(), "PF00001");
        assert_eq!(acc.clone().into_string(), "PF00001");
        // Wrong prefix: catches `is_pfam_accession -> true` mutation.
        assert!(PfamAccession::new("XX00001").is_err());
        // Right prefix, non-digit tail: catches && -> || in is_pfam_accession.
        assert!(PfamAccession::new("PFabcde").is_err());
        // Right prefix, wrong length: catches len-check mutations.
        assert!(PfamAccession::new("PF000001").is_err());
    }

    #[test]
    fn tax_id_exposes_inner_value() {
        let tax = TaxId::new(3197);
        assert_eq!(tax.get(), 3197);
        assert_eq!(tax.to_string(), "3197");
    }
}
