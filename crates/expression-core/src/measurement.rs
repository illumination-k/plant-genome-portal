use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use genome_core::GeneId;

use crate::ids::SampleId;
use crate::unit::ExpressionUnit;
use crate::value::ExpressionValue;

/// A single expression measurement: the level of one gene in one sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ExpressionMeasurement {
    pub gene_id: GeneId,
    pub sample_id: SampleId,
    pub value: f64,
    pub unit: ExpressionUnit,
}

impl ExpressionMeasurement {
    pub fn new(gene_id: GeneId, sample_id: SampleId, value: ExpressionValue) -> Self {
        Self {
            gene_id,
            sample_id,
            value: value.value,
            unit: value.unit,
        }
    }
}
