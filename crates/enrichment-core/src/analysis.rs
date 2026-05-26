use std::collections::HashSet;
use std::hash::Hash;

use crate::error::EnrichmentError;
use crate::fdr::benjamini_hochberg;
use crate::hypergeometric::hypergeometric_upper_tail;
use crate::result::EnrichmentResult;

/// Input bundle for [`run_enrichment`].
///
/// Generic over the term type `T` (e.g. a GO term ID) and item type `G`
/// (e.g. a gene ID). The `term_to_items` mapping is the only source of
/// truth for which items belong to which term — items missing from the
/// population are dropped from the contingency table before testing, so
/// callers may pass annotations for items that are not in the
/// background without distorting results.
#[derive(Debug, Clone)]
pub struct EnrichmentInput<'a, T, G> {
    pub study: &'a HashSet<G>,
    pub population: &'a HashSet<G>,
    pub term_to_items: &'a [(T, HashSet<G>)],
}

/// Knobs that filter or trim the result set.
#[derive(Debug, Clone, Copy)]
pub struct EnrichmentOptions {
    /// Skip terms whose population hit count is below this threshold.
    /// Useful for dropping sparsely-annotated terms whose `p`-values are
    /// statistically unstable. Default: `2`.
    pub min_population_hits: u64,
}

impl Default for EnrichmentOptions {
    fn default() -> Self {
        Self {
            min_population_hits: 2,
        }
    }
}

/// Run one-sided over-representation analysis for every term in `input`.
///
/// Steps:
/// 1. Restrict the study set to items that also belong to the population.
/// 2. For each term, build the 2 x 2 contingency table restricted to the
///    population, dropping terms that fall below
///    [`EnrichmentOptions::min_population_hits`].
/// 3. Compute the upper-tail hypergeometric `p`-value and fold enrichment.
/// 4. Apply Benjamini-Hochberg FDR across the surviving terms.
///
/// The returned vector is sorted ascending by `p_value` (then by
/// `population_hits` descending as a stable tiebreaker).
pub fn run_enrichment<T, G>(
    input: EnrichmentInput<'_, T, G>,
    options: EnrichmentOptions,
) -> Result<Vec<EnrichmentResult<T>>, EnrichmentError>
where
    T: Clone,
    G: Eq + Hash,
{
    if input.population.is_empty() {
        return Err(EnrichmentError::EmptyPopulation);
    }

    let population_size = input.population.len() as u64;
    let study_in_population: HashSet<&G> = input
        .study
        .iter()
        .filter(|item| input.population.contains(*item))
        .collect();
    let study_size = study_in_population.len() as u64;

    let partial = tested_terms(
        input,
        options,
        &study_in_population,
        population_size,
        study_size,
    )?;

    let p_values: Vec<f64> = partial.iter().map(|(r, _)| r.p_value).collect();
    let q_values = benjamini_hochberg(&p_values);
    let mut results: Vec<EnrichmentResult<T>> = partial
        .into_iter()
        .zip(q_values)
        .map(|((mut r, _), q)| {
            r.q_value = q;
            r
        })
        .collect();

    results.sort_by(|a, b| {
        a.p_value
            .partial_cmp(&b.p_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.population_hits.cmp(&a.population_hits))
    });

    Ok(results)
}

fn tested_terms<T, G>(
    input: EnrichmentInput<'_, T, G>,
    options: EnrichmentOptions,
    study_in_population: &HashSet<&G>,
    population_size: u64,
    study_size: u64,
) -> Result<Vec<(EnrichmentResult<T>, ())>, EnrichmentError>
where
    T: Clone,
    G: Eq + Hash,
{
    let mut partial = Vec::new();
    for (term, items) in input.term_to_items {
        let Some(result) = tested_term(
            term,
            items,
            input.population,
            study_in_population,
            options,
            population_size,
            study_size,
        )?
        else {
            continue;
        };
        partial.push((result, ()));
    }
    Ok(partial)
}

fn tested_term<T, G>(
    term: &T,
    items: &HashSet<G>,
    population: &HashSet<G>,
    study_in_population: &HashSet<&G>,
    options: EnrichmentOptions,
    population_size: u64,
    study_size: u64,
) -> Result<Option<EnrichmentResult<T>>, EnrichmentError>
where
    T: Clone,
    G: Eq + Hash,
{
    let (population_hits, study_hits) = count_hits(items, population, study_in_population);
    if population_hits < options.min_population_hits {
        return Ok(None);
    }

    let p_value =
        hypergeometric_upper_tail(population_size, population_hits, study_size, study_hits)?;
    Ok(Some(EnrichmentResult {
        term: term.clone(),
        study_hits,
        study_size,
        population_hits,
        population_size,
        fold_enrichment: EnrichmentResult::<T>::compute_fold(
            study_hits,
            study_size,
            population_hits,
            population_size,
        ),
        p_value,
        q_value: f64::NAN,
    }))
}

fn count_hits<'a, G>(
    items: &'a HashSet<G>,
    population: &'a HashSet<G>,
    study_in_population: &HashSet<&'a G>,
) -> (u64, u64)
where
    G: Eq + Hash,
{
    let mut population_hits = 0;
    let mut study_hits = 0;
    for item in items {
        if population.contains(item) {
            population_hits += 1;
            if study_in_population.contains(item) {
                study_hits += 1;
            }
        }
    }
    (population_hits, study_hits)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn population_of(genes: &[&'static str]) -> HashSet<&'static str> {
        genes.iter().copied().collect()
    }

    #[test]
    fn empty_population_is_an_error() {
        let study: HashSet<&str> = HashSet::new();
        let population: HashSet<&str> = HashSet::new();
        let map: Vec<(&str, HashSet<&str>)> = Vec::new();
        let err = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, EnrichmentError::EmptyPopulation));
    }

    #[test]
    fn study_items_outside_population_are_dropped() {
        // Population = {A, B, C, D}; study claims {A, X, Y}.
        // Only A survives the restriction, so study_size = 1.
        let population = population_of(&["A", "B", "C", "D"]);
        let study = population_of(&["A", "X", "Y"]);
        let term_genes = population_of(&["A", "B"]);
        let map = vec![("GO:1", term_genes)];

        let results = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions::default(),
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].study_size, 1);
        assert_eq!(results[0].study_hits, 1);
        assert_eq!(results[0].population_hits, 2);
        assert_eq!(results[0].population_size, 4);
    }

    #[test]
    fn min_population_hits_filters_sparse_terms() {
        let population = population_of(&["A", "B", "C", "D", "E"]);
        let study = population_of(&["A"]);
        let map = vec![
            ("GO:rare", population_of(&["A"])),        // 1 pop hit
            ("GO:common", population_of(&["A", "B"])), // 2 pop hits
        ];
        let results = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions {
                min_population_hits: 2,
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].term, "GO:common");
    }

    #[test]
    fn enriched_term_ranks_above_unenriched_term() {
        // Population of 20 genes, study = 5 of them. One term covers
        // the entire study and nothing else; another term covers 5
        // unrelated genes.
        let population: HashSet<&str> = (0..20)
            .map(|i| match i {
                0 => "g0",
                1 => "g1",
                2 => "g2",
                3 => "g3",
                4 => "g4",
                5 => "g5",
                6 => "g6",
                7 => "g7",
                8 => "g8",
                9 => "g9",
                10 => "g10",
                11 => "g11",
                12 => "g12",
                13 => "g13",
                14 => "g14",
                15 => "g15",
                16 => "g16",
                17 => "g17",
                18 => "g18",
                _ => "g19",
            })
            .collect();
        let study = population_of(&["g0", "g1", "g2", "g3", "g4"]);
        let map = vec![
            ("hit", population_of(&["g0", "g1", "g2", "g3", "g4"])),
            ("miss", population_of(&["g10", "g11", "g12", "g13", "g14"])),
        ];

        let results = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions::default(),
        )
        .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].term, "hit");
        assert!(results[0].p_value < results[1].p_value);
        assert!(results[0].fold_enrichment.unwrap() > 1.0);
    }

    #[test]
    fn fold_enrichment_is_none_when_term_has_no_population_hits_after_filter() {
        // A term whose only annotated item is outside the population
        // is filtered out before we record a result, so it can't appear
        // with population_hits = 0. Verify the filter actually fires.
        let population = population_of(&["A", "B"]);
        let study = population_of(&["A"]);
        let term_outside = population_of(&["Z"]); // 0 population hits
        let map = vec![("GO:1", term_outside)];

        let results = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions {
                min_population_hits: 1,
            },
        )
        .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn q_values_are_assigned_in_input_order() {
        // Two terms with the same p-value should both receive the same
        // q-value, and both q-values must be populated (not NaN, which
        // is the initial sentinel).
        let population = population_of(&["A", "B", "C", "D"]);
        let study = population_of(&["A", "B"]);
        let map = vec![
            ("T1", population_of(&["A", "B"])),
            ("T2", population_of(&["A", "B"])),
        ];
        let results = run_enrichment(
            EnrichmentInput {
                study: &study,
                population: &population,
                term_to_items: &map,
            },
            EnrichmentOptions::default(),
        )
        .unwrap();
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(!r.q_value.is_nan(), "q_value left as sentinel: {r:?}");
        }
        assert!((results[0].q_value - results[1].q_value).abs() < 1e-12);
    }
}
