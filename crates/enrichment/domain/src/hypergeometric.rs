use statrs::distribution::{DiscreteCDF, Hypergeometric};

use crate::error::EnrichmentError;

/// One-sided upper-tail hypergeometric probability `P(X >= observed)`.
///
/// Models drawing `draws` items without replacement from a population of
/// `population` items, of which `successes` are labelled as belonging to
/// the term under test. The returned value is the probability of seeing
/// at least `observed` successes in the draw — the standard Fisher
/// one-sided over-representation `p`-value.
///
/// # Constraints
///
/// * `successes <= population` and `draws <= population`.
/// * `observed` must lie inside the feasible support
///   `[max(0, draws + successes - population), min(draws, successes)]`.
///   Observed counts outside this range collapse to the natural limit
///   (zero observed -> `p = 1`, observed above the max -> `p = 0`).
pub fn hypergeometric_upper_tail(
    population: u64,
    successes: u64,
    draws: u64,
    observed: u64,
) -> Result<f64, EnrichmentError> {
    // P(X >= 0) is always 1 — short-circuit before constructing the
    // distribution so we never feed `sf(-1)` into statrs.
    if observed == 0 {
        return Ok(1.0);
    }

    // statrs validates `successes <= population` and `draws <= population`
    // when constructing the distribution; surface that as our domain error
    // rather than duplicating the check.
    let dist = Hypergeometric::new(population, successes, draws).map_err(|_| {
        EnrichmentError::InvalidHypergeometric {
            population,
            successes,
            draws,
        }
    })?;

    // statrs::sf(x) = P(X > x), so P(X >= observed) = sf(observed - 1).
    // For `observed` above `min(successes, draws)` statrs returns 0 (the
    // value is past the support), which is exactly the answer we want.
    Ok(dist.sf(observed - 1).clamp(0.0, 1.0))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn observed_zero_returns_one() {
        let p = hypergeometric_upper_tail(100, 10, 20, 0).unwrap();
        assert_eq!(p, 1.0);
    }

    #[test]
    fn observed_above_max_returns_zero() {
        // Drawing 20 from a population of 100 with 10 successes: max
        // possible observed is min(20, 10) = 10.
        let p = hypergeometric_upper_tail(100, 10, 20, 11).unwrap();
        assert_eq!(p, 0.0);
    }

    #[test]
    fn observed_at_max_returns_small_positive_probability() {
        // observed = min(successes, draws) is the largest feasible value,
        // attainable with non-zero (but small) probability. P(X = 10) =
        // C(10,10) * C(90,10) / C(100,20) ~= 1.07e-7. Anything that
        // collapses this case to 0 (e.g. a `>` -> `>=` mutation on the
        // out-of-support short-circuit) would fail this assertion.
        let p = hypergeometric_upper_tail(100, 10, 20, 10).unwrap();
        assert!(p > 0.0 && p < 1e-6, "got {p}");
    }

    #[test]
    fn successes_equal_to_population_succeeds() {
        // When every population item is a "success" and we draw any
        // positive subset, we always see `draws` successes — p = 1 at
        // observed = draws. This kills the `successes > population`
        // -> `successes == population` and `>=` mutants on the validation
        // line (which would otherwise reject this valid input).
        let p = hypergeometric_upper_tail(10, 10, 3, 3).unwrap();
        assert!((p - 1.0).abs() < 1e-12, "got {p}");
    }

    #[test]
    fn draws_equal_to_population_succeeds() {
        // Drawing the entire population: every success is observed.
        // P(X >= successes) = 1. Kills the matching mutants on the
        // `draws > population` side of the validation.
        let p = hypergeometric_upper_tail(10, 3, 10, 3).unwrap();
        assert!((p - 1.0).abs() < 1e-12, "got {p}");
    }

    #[test]
    fn matches_known_fisher_value() {
        // 2x2 table:
        //   in study & in term     = 5
        //   in study & not in term = 15
        //   not study & in term    = 5
        //   not study & not in term = 75
        // i.e. population=100, successes=10 (in term), draws=20 (study), observed=5.
        // Upper-tail p-value via Fisher's exact (R: phyper(4, 10, 90, 20,
        // lower.tail=FALSE); scipy: hypergeom.sf(4, 100, 10, 20)).
        let p = hypergeometric_upper_tail(100, 10, 20, 5).unwrap();
        assert!(approx(p, 0.02546454, 1e-6), "got {p}");
    }

    #[test]
    fn matches_simple_hand_computation() {
        // Urn of 10 with 5 "successes"; draw 3. P(X >= 1) = 1 - P(X = 0)
        //   P(X = 0) = C(5,0) * C(5,3) / C(10,3) = 10/120 = 1/12
        //   P(X >= 1) = 11/12 ~= 0.91666...
        let p = hypergeometric_upper_tail(10, 5, 3, 1).unwrap();
        assert!(approx(p, 11.0 / 12.0, 1e-12), "got {p}");
    }

    #[test]
    fn p_is_one_at_minimum_support() {
        // observed = max(0, draws + successes - population) is the smallest
        // feasible observed count, so P(X >= that) must equal 1.
        // population=100, successes=80, draws=80 -> min observed = 60.
        let p = hypergeometric_upper_tail(100, 80, 80, 60).unwrap();
        assert!(approx(p, 1.0, 1e-12), "got {p}");
    }

    #[test]
    fn rejects_successes_above_population() {
        let err = hypergeometric_upper_tail(10, 11, 5, 1).unwrap_err();
        assert!(matches!(err, EnrichmentError::InvalidHypergeometric { .. }));
    }

    #[test]
    fn rejects_draws_above_population() {
        let err = hypergeometric_upper_tail(10, 5, 11, 1).unwrap_err();
        assert!(matches!(err, EnrichmentError::InvalidHypergeometric { .. }));
    }

    #[test]
    fn result_is_clamped_into_unit_interval() {
        // Floating-point round-off can push the survival function slightly
        // negative or above 1. Hit a parameter set that lands near the
        // boundary and verify we never escape [0, 1].
        let p = hypergeometric_upper_tail(50, 25, 25, 25).unwrap();
        assert!((0.0..=1.0).contains(&p), "got {p}");
    }
}
