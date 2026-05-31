use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::EpigenomeError;

macro_rules! geo_accession {
    (
        $name:ident,
        prefix = $prefix:literal,
        min_digits = $min_digits:expr,
        error = $error:ident,
        schema_kind = $schema_kind:literal,
    ) => {
        #[doc = concat!("A GEO ", $schema_kind, " accession.")]
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EpigenomeError> {
                let value = value.into();
                let Some(tail) = value.strip_prefix($prefix) else {
                    return Err(EpigenomeError::$error(value));
                };
                if tail.len() < $min_digits || !tail.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(EpigenomeError::$error(value));
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
            type Err = EpigenomeError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

geo_accession!(
    GeoSeriesAccession,
    prefix = "GSE",
    min_digits = 1,
    error = InvalidGeoSeriesAccession,
    schema_kind = "Series",
);

geo_accession!(
    GeoSampleAccession,
    prefix = "GSM",
    min_digits = 1,
    error = InvalidGeoSampleAccession,
    schema_kind = "Sample",
);

macro_rules! non_empty_string {
    ($name:ident, $kind:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, EpigenomeError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(EpigenomeError::EmptyIdentifier { kind: $kind });
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
            type Err = EpigenomeError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

non_empty_string!(ExperimentId, "experiment id");
non_empty_string!(Target, "target");
non_empty_string!(Antibody, "antibody");

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("GSE1")]
    #[case("GSE123456")]
    fn geo_series_accepts(#[case] value: &str) {
        let id = GeoSeriesAccession::new(value).unwrap();
        assert_eq!(id.as_str(), value);
        assert_eq!(id.to_string(), value);
        let parsed: GeoSeriesAccession = value.parse().unwrap();
        assert_eq!(parsed.into_string(), value);
    }

    #[rstest]
    #[case("")]
    #[case("gse1")]
    #[case("GSE")]
    #[case("GSM1")]
    #[case("GSEabc")]
    fn geo_series_rejects(#[case] value: &str) {
        assert!(GeoSeriesAccession::new(value).is_err());
    }

    #[rstest]
    #[case("GSM1")]
    #[case("GSM7890123")]
    fn geo_sample_accepts(#[case] value: &str) {
        let id = GeoSampleAccession::new(value).unwrap();
        assert_eq!(id.as_str(), value);
    }

    #[rstest]
    #[case("")]
    #[case("GSE1")]
    #[case("GSM")]
    #[case("GSMxyz")]
    fn geo_sample_rejects(#[case] value: &str) {
        assert!(GeoSampleAccession::new(value).is_err());
    }

    #[rstest]
    #[case("mp_h3k4me3_thallus_rep1")]
    #[case("any-string")]
    fn experiment_id_accepts(#[case] value: &str) {
        let id = ExperimentId::new(value).unwrap();
        assert_eq!(id.as_str(), value);
    }

    #[rstest]
    #[case("")]
    #[case("   ")]
    fn experiment_id_rejects_empty(#[case] value: &str) {
        let err = ExperimentId::new(value).unwrap_err();
        assert!(matches!(
            err,
            EpigenomeError::EmptyIdentifier {
                kind: "experiment id"
            }
        ));
    }

    #[test]
    fn target_and_antibody_reject_blank() {
        assert!(Target::new("").is_err());
        assert!(Antibody::new("   ").is_err());
        assert_eq!(Target::new("H3K4me3").unwrap().as_str(), "H3K4me3");
        assert_eq!(Antibody::new("ab12345").unwrap().as_str(), "ab12345");
    }
}
