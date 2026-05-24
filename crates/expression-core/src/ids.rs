use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::error::ExpressionError;

macro_rules! string_id {
    ($name:ident, $kind:literal) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ExpressionError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(ExpressionError::EmptyIdentifier { kind: $kind });
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

string_id!(SampleId, "sample id");
string_id!(ExperimentId, "experiment id");

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn sample_id_round_trips() {
        let id = SampleId::new("SAMN0001").unwrap();
        assert_eq!(id.as_str(), "SAMN0001");
        assert_eq!(id.to_string(), "SAMN0001");
        assert_eq!(id.clone().into_string(), "SAMN0001");
    }

    #[test]
    fn empty_sample_id_is_rejected() {
        assert!(matches!(
            SampleId::new(""),
            Err(ExpressionError::EmptyIdentifier { kind: "sample id" })
        ));
        assert!(matches!(
            SampleId::new("   "),
            Err(ExpressionError::EmptyIdentifier { kind: "sample id" })
        ));
    }

    #[test]
    fn empty_experiment_id_is_rejected() {
        assert!(matches!(
            ExperimentId::new(""),
            Err(ExpressionError::EmptyIdentifier {
                kind: "experiment id"
            })
        ));
    }

    #[test]
    fn ids_parse_from_str() {
        let sample: SampleId = "S1".parse().unwrap();
        let experiment: ExperimentId = "E1".parse().unwrap();
        assert_eq!(sample.as_str(), "S1");
        assert_eq!(experiment.as_str(), "E1");
    }
}
