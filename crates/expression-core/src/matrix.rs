use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use genome_core::{AssemblyAccession, GeneId};

use crate::error::ExpressionError;
use crate::ids::SampleId;
use crate::unit::ExpressionUnit;

/// A dense gene × sample expression matrix.
///
/// Values are stored row-major: the value for `(gene_idx, sample_idx)` lives at
/// `values[gene_idx * sample_count + sample_idx]`. All entries share the same
/// `unit` — heterogeneous units belong in separate matrices.
///
/// `NaN` is allowed in `values` to represent missing measurements; callers
/// that need stricter validation should construct via [`ExpressionValue`] and
/// then assemble the matrix themselves.
///
/// [`ExpressionValue`]: crate::value::ExpressionValue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ExpressionMatrix {
    pub assembly_accession: AssemblyAccession,
    pub unit: ExpressionUnit,
    pub gene_ids: Vec<GeneId>,
    pub sample_ids: Vec<SampleId>,
    pub values: Vec<f64>,
}

impl ExpressionMatrix {
    pub fn new(
        assembly_accession: AssemblyAccession,
        unit: ExpressionUnit,
        gene_ids: Vec<GeneId>,
        sample_ids: Vec<SampleId>,
        values: Vec<f64>,
    ) -> Result<Self, ExpressionError> {
        let expected = gene_ids.len().saturating_mul(sample_ids.len());
        if values.len() != expected {
            return Err(ExpressionError::DimensionMismatch {
                expected,
                actual: values.len(),
            });
        }
        Ok(Self {
            assembly_accession,
            unit,
            gene_ids,
            sample_ids,
            values,
        })
    }

    pub fn gene_count(&self) -> usize {
        self.gene_ids.len()
    }

    pub fn sample_count(&self) -> usize {
        self.sample_ids.len()
    }

    /// Returns the value at `(gene_idx, sample_idx)`, or `None` if either
    /// index is out of bounds.
    pub fn value(&self, gene_idx: usize, sample_idx: usize) -> Option<f64> {
        if gene_idx >= self.gene_count() || sample_idx >= self.sample_count() {
            return None;
        }
        let offset = gene_idx
            .checked_mul(self.sample_count())?
            .checked_add(sample_idx)?;
        self.values.get(offset).copied()
    }

    /// Returns the row (all samples) for the given gene index, or `None` if
    /// the gene index is out of bounds.
    pub fn gene_row(&self, gene_idx: usize) -> Option<&[f64]> {
        if gene_idx >= self.gene_count() {
            return None;
        }
        let n = self.sample_count();
        let start = gene_idx.checked_mul(n)?;
        let end = start.checked_add(n)?;
        self.values.get(start..end)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new("GCA_037833805.1").unwrap()
    }

    fn gene(id: &str) -> GeneId {
        GeneId::new(id).unwrap()
    }

    fn sample(id: &str) -> SampleId {
        SampleId::new(id).unwrap()
    }

    #[test]
    fn rejects_mismatched_dimensions() {
        let err = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![gene("g1"), gene("g2")],
            vec![sample("s1"), sample("s2"), sample("s3")],
            vec![0.0; 5],
        )
        .unwrap_err();
        assert_eq!(
            err,
            ExpressionError::DimensionMismatch {
                expected: 6,
                actual: 5
            }
        );
    }

    #[test]
    fn accepts_empty_matrix() {
        let m =
            ExpressionMatrix::new(assembly(), ExpressionUnit::Tpm, vec![], vec![], vec![]).unwrap();
        assert_eq!(m.gene_count(), 0);
        assert_eq!(m.sample_count(), 0);
        assert!(m.value(0, 0).is_none());
        assert!(m.gene_row(0).is_none());
    }

    #[test]
    fn row_major_value_lookup() {
        let m = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![gene("g1"), gene("g2")],
            vec![sample("s1"), sample("s2"), sample("s3")],
            vec![
                10.0, 20.0, 30.0, // g1
                40.0, 50.0, 60.0, // g2
            ],
        )
        .unwrap();

        assert_eq!(m.value(0, 0), Some(10.0));
        assert_eq!(m.value(0, 2), Some(30.0));
        assert_eq!(m.value(1, 0), Some(40.0));
        assert_eq!(m.value(1, 2), Some(60.0));
        assert_eq!(m.value(2, 0), None);
        assert_eq!(m.value(0, 3), None);

        assert_eq!(m.gene_row(0), Some([10.0, 20.0, 30.0].as_slice()));
        assert_eq!(m.gene_row(1), Some([40.0, 50.0, 60.0].as_slice()));
        assert_eq!(m.gene_row(2), None);
    }
}
