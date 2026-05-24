use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::ExpressionError;

/// Returns `true` if `value` starts with one of the prefixes from
/// `allowed_prefixes` and the remainder is at least `min_digits` ASCII digits.
fn has_prefix_and_digits(value: &str, allowed_prefixes: &[&str], min_digits: usize) -> bool {
    let Some(prefix) = allowed_prefixes
        .iter()
        .find(|prefix| value.starts_with(*prefix))
    else {
        return false;
    };
    let tail = &value[prefix.len()..];
    tail.len() >= min_digits && tail.bytes().all(|byte| byte.is_ascii_digit())
}

macro_rules! sra_accession {
    (
        $name:ident,
        prefixes = [$($prefix:literal),+ $(,)?],
        min_digits = $min_digits:expr,
        error = $error:ident,
        schema_kind = $schema_kind:literal,
    ) => {
        #[doc = concat!("An SRA / INSDC ", $schema_kind, " accession.")]
        #[doc = ""]
        #[doc = concat!("Format: one of `", stringify!($($prefix),+), "` followed by at least ",
            stringify!($min_digits), " ASCII digits.")]
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ExpressionError> {
                let value = value.into();
                if !has_prefix_and_digits(&value, &[$($prefix),+], $min_digits) {
                    return Err(ExpressionError::$error(value));
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
            type Err = ExpressionError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

sra_accession!(
    SraRunAccession,
    prefixes = ["SRR", "ERR", "DRR"],
    min_digits = 6,
    error = InvalidSraRunAccession,
    schema_kind = "SRA Run",
);

sra_accession!(
    SraExperimentAccession,
    prefixes = ["SRX", "ERX", "DRX"],
    min_digits = 6,
    error = InvalidSraExperimentAccession,
    schema_kind = "SRA Experiment",
);

sra_accession!(
    SraStudyAccession,
    prefixes = ["SRP", "ERP", "DRP"],
    min_digits = 6,
    error = InvalidSraStudyAccession,
    schema_kind = "SRA Study",
);

sra_accession!(
    BioSampleAccession,
    prefixes = ["SAMN", "SAMEA", "SAMD"],
    min_digits = 1,
    error = InvalidBioSampleAccession,
    schema_kind = "BioSample",
);

sra_accession!(
    BioProjectAccession,
    prefixes = ["PRJNA", "PRJEB", "PRJDB"],
    min_digits = 1,
    error = InvalidBioProjectAccession,
    schema_kind = "BioProject",
);

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn sra_run_accession_accepts_all_three_archives() {
        assert!(SraRunAccession::new("SRR000001").is_ok());
        assert!(SraRunAccession::new("ERR1234567").is_ok());
        assert!(SraRunAccession::new("DRR000001").is_ok());
        // 10+ digits are common in modern entries.
        assert!(SraRunAccession::new("SRR1234567890").is_ok());
    }

    #[test]
    fn sra_run_accession_rejects_wrong_prefix_or_short_digits() {
        // Wrong prefix.
        assert!(SraRunAccession::new("XRR000001").is_err());
        assert!(SraRunAccession::new("SRX000001").is_err());
        // Right prefix, too few digits.
        assert!(SraRunAccession::new("SRR123").is_err());
        // Right prefix, non-digit tail.
        assert!(SraRunAccession::new("SRRabcdef").is_err());
        // Empty.
        assert!(SraRunAccession::new("").is_err());
    }

    #[test]
    fn sra_experiment_accession_validates_format() {
        assert!(SraExperimentAccession::new("SRX000001").is_ok());
        assert!(SraExperimentAccession::new("ERX123456").is_ok());
        assert!(SraExperimentAccession::new("DRX999999").is_ok());
        assert!(SraExperimentAccession::new("SRR000001").is_err());
    }

    #[test]
    fn sra_study_accession_validates_format() {
        assert!(SraStudyAccession::new("SRP000001").is_ok());
        assert!(SraStudyAccession::new("ERP000001").is_ok());
        assert!(SraStudyAccession::new("DRP000001").is_ok());
        assert!(SraStudyAccession::new("SRX000001").is_err());
    }

    #[test]
    fn biosample_accession_validates_format() {
        assert!(BioSampleAccession::new("SAMN12345678").is_ok());
        assert!(BioSampleAccession::new("SAMEA1234567").is_ok());
        assert!(BioSampleAccession::new("SAMD00012345").is_ok());
        // Wrong prefix.
        assert!(BioSampleAccession::new("SAM12345").is_err());
        assert!(BioSampleAccession::new("PRJNA1").is_err());
        // Non-digit tail.
        assert!(BioSampleAccession::new("SAMNabcd").is_err());
    }

    #[test]
    fn bioproject_accession_validates_format() {
        assert!(BioProjectAccession::new("PRJNA1").is_ok());
        assert!(BioProjectAccession::new("PRJEB12345").is_ok());
        assert!(BioProjectAccession::new("PRJDB6789").is_ok());
        assert!(BioProjectAccession::new("PRJ123").is_err());
        assert!(BioProjectAccession::new("SAMN1").is_err());
    }

    #[test]
    fn round_trip_via_as_str_into_string_display() {
        let run = SraRunAccession::new("SRR000001").unwrap();
        assert_eq!(run.as_str(), "SRR000001");
        assert_eq!(run.to_string(), "SRR000001");
        assert_eq!(run.clone().into_string(), "SRR000001");
    }

    #[test]
    fn ids_parse_from_str() {
        let run: SraRunAccession = "SRR000001".parse().unwrap();
        let project: BioProjectAccession = "PRJNA1".parse().unwrap();
        assert_eq!(run.as_str(), "SRR000001");
        assert_eq!(project.as_str(), "PRJNA1");
    }
}
