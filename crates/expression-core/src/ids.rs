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
    use rstest::rstest;

    #[derive(Clone, Copy, Debug)]
    enum AccessionKind {
        SraRun,
        SraExperiment,
        SraStudy,
        BioSample,
        BioProject,
    }

    impl AccessionKind {
        fn assert_accepts(self, value: &str) {
            match self {
                Self::SraRun => {
                    let id = SraRunAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                Self::SraExperiment => {
                    let id = SraExperimentAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                Self::SraStudy => {
                    let id = SraStudyAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                Self::BioSample => {
                    let id = BioSampleAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
                Self::BioProject => {
                    let id = BioProjectAccession::new(value).unwrap();
                    assert_eq!(id.as_str(), value);
                    assert_eq!(id.to_string(), value);
                    assert_eq!(id.clone().into_string(), value);
                }
            }
        }

        fn assert_rejects(self, value: &str) {
            let is_err = match self {
                Self::SraRun => SraRunAccession::new(value).is_err(),
                Self::SraExperiment => SraExperimentAccession::new(value).is_err(),
                Self::SraStudy => SraStudyAccession::new(value).is_err(),
                Self::BioSample => BioSampleAccession::new(value).is_err(),
                Self::BioProject => BioProjectAccession::new(value).is_err(),
            };
            assert!(is_err, "{self:?} accepted invalid value {value:?}");
        }

        fn assert_parses(self, value: &str) {
            match self {
                Self::SraRun => {
                    let id: SraRunAccession = value.parse().unwrap();
                    assert_eq!(id.as_str(), value);
                }
                Self::SraExperiment => {
                    let id: SraExperimentAccession = value.parse().unwrap();
                    assert_eq!(id.as_str(), value);
                }
                Self::SraStudy => {
                    let id: SraStudyAccession = value.parse().unwrap();
                    assert_eq!(id.as_str(), value);
                }
                Self::BioSample => {
                    let id: BioSampleAccession = value.parse().unwrap();
                    assert_eq!(id.as_str(), value);
                }
                Self::BioProject => {
                    let id: BioProjectAccession = value.parse().unwrap();
                    assert_eq!(id.as_str(), value);
                }
            }
        }
    }

    #[rstest]
    #[case::sra_run_sra(AccessionKind::SraRun, "SRR000001")]
    #[case::sra_run_ena(AccessionKind::SraRun, "ERR1234567")]
    #[case::sra_run_dra(AccessionKind::SraRun, "DRR000001")]
    #[case::sra_run_modern_digits(AccessionKind::SraRun, "SRR1234567890")]
    #[case::sra_experiment_sra(AccessionKind::SraExperiment, "SRX000001")]
    #[case::sra_experiment_ena(AccessionKind::SraExperiment, "ERX123456")]
    #[case::sra_experiment_dra(AccessionKind::SraExperiment, "DRX999999")]
    #[case::sra_study_sra(AccessionKind::SraStudy, "SRP000001")]
    #[case::sra_study_ena(AccessionKind::SraStudy, "ERP000001")]
    #[case::sra_study_dra(AccessionKind::SraStudy, "DRP000001")]
    #[case::biosample_ncbi(AccessionKind::BioSample, "SAMN12345678")]
    #[case::biosample_ena(AccessionKind::BioSample, "SAMEA1234567")]
    #[case::biosample_dra(AccessionKind::BioSample, "SAMD00012345")]
    #[case::bioproject_ncbi(AccessionKind::BioProject, "PRJNA1")]
    #[case::bioproject_ena(AccessionKind::BioProject, "PRJEB12345")]
    #[case::bioproject_dra(AccessionKind::BioProject, "PRJDB6789")]
    fn accessions_accept_valid_values(#[case] kind: AccessionKind, #[case] value: &str) {
        kind.assert_accepts(value);
    }

    #[rstest]
    #[case::sra_run_wrong_prefix(AccessionKind::SraRun, "XRR000001")]
    #[case::sra_run_experiment_prefix(AccessionKind::SraRun, "SRX000001")]
    #[case::sra_run_short_digits(AccessionKind::SraRun, "SRR123")]
    #[case::sra_run_non_digits(AccessionKind::SraRun, "SRRabcdef")]
    #[case::sra_run_empty(AccessionKind::SraRun, "")]
    #[case::sra_experiment_run_prefix(AccessionKind::SraExperiment, "SRR000001")]
    #[case::sra_study_experiment_prefix(AccessionKind::SraStudy, "SRX000001")]
    #[case::biosample_short_prefix(AccessionKind::BioSample, "SAM12345")]
    #[case::biosample_project_prefix(AccessionKind::BioSample, "PRJNA1")]
    #[case::biosample_non_digits(AccessionKind::BioSample, "SAMNabcd")]
    #[case::bioproject_short_prefix(AccessionKind::BioProject, "PRJ123")]
    #[case::bioproject_sample_prefix(AccessionKind::BioProject, "SAMN1")]
    fn accessions_reject_invalid_values(#[case] kind: AccessionKind, #[case] value: &str) {
        kind.assert_rejects(value);
    }

    #[rstest]
    #[case::sra_run(AccessionKind::SraRun, "SRR000001")]
    #[case::sra_experiment(AccessionKind::SraExperiment, "SRX000001")]
    #[case::sra_study(AccessionKind::SraStudy, "SRP000001")]
    #[case::biosample(AccessionKind::BioSample, "SAMN12345678")]
    #[case::bioproject(AccessionKind::BioProject, "PRJNA1")]
    fn accessions_parse_from_str(#[case] kind: AccessionKind, #[case] value: &str) {
        kind.assert_parses(value);
    }
}
