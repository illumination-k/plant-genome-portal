use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Per-term enrichment statistic.
///
/// All counts are restricted to the population (background) — items in
/// the raw study set that do not appear in the population are dropped
/// before the contingency table is built (see [`crate::run_enrichment`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct EnrichmentResult<T> {
    /// The term being tested (e.g. a GO term, Pfam accession, ...).
    pub term: T,
    /// Items in the study set that are annotated with `term`.
    pub study_hits: u64,
    /// Items in the study set (after restricting to the population).
    pub study_size: u64,
    /// Items in the population that are annotated with `term`.
    pub population_hits: u64,
    /// Total population size.
    pub population_size: u64,
    /// `(study_hits / study_size) / (population_hits / population_size)`.
    ///
    /// `None` when the term has no population hits (the ratio is undefined).
    pub fold_enrichment: Option<f64>,
    /// One-sided hypergeometric `P(X >= study_hits)`.
    pub p_value: f64,
    /// Benjamini-Hochberg-adjusted `p`-value across the returned terms.
    pub q_value: f64,
}

impl<T> EnrichmentResult<T> {
    pub(crate) fn compute_fold(
        study_hits: u64,
        study_size: u64,
        population_hits: u64,
        population_size: u64,
    ) -> Option<f64> {
        if population_hits == 0 || study_size == 0 {
            return None;
        }
        let observed = (study_hits as f64) / (study_size as f64);
        let expected = (population_hits as f64) / (population_size as f64);
        Some(observed / expected)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn fold_enrichment_handles_zero_denominator() {
        assert!(EnrichmentResult::<&str>::compute_fold(0, 10, 0, 100).is_none());
        assert!(EnrichmentResult::<&str>::compute_fold(0, 0, 5, 100).is_none());
    }

    #[test]
    fn fold_enrichment_basic_ratio() {
        // 5/20 vs 10/100 = 0.25 / 0.10 = 2.5
        let f = EnrichmentResult::<&str>::compute_fold(5, 20, 10, 100).unwrap();
        assert!((f - 2.5).abs() < 1e-12, "got {f}");
    }
}
