//! Pairwise and set-level semantic similarity for GO terms.
//!
//! Four pairwise metrics are exposed:
//!
//! * [`SimilarityMethod::Resnik`] — `IC(MICA)`
//! * [`SimilarityMethod::Lin`] — `2·IC(MICA) / (IC(t1) + IC(t2))`
//! * [`SimilarityMethod::JiangConrath`] — `1 / (1 + dist)` where
//!   `dist = IC(t1) + IC(t2) - 2·IC(MICA)`
//! * [`wang`] — structural Wang 2007 measure (no IC required)
//!
//! Set-level similarity ([`set_similarity`]) is computed from any pairwise
//! score using one of [`SetAggregator::BestMatchAverage`],
//! [`SetAggregator::Max`], or [`SetAggregator::Average`].

use std::collections::HashMap;

use genome_domain::GoTermId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dag::GoDag;
use crate::ic::InformationContent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityMethod {
    Resnik,
    Lin,
    JiangConrath,
}

/// MICA (Most Informative Common Ancestor) of two terms together with its
/// IC. `None` if the terms share no ancestor with a known IC.
pub fn mica<I: InformationContent>(
    dag: &GoDag,
    ic: &I,
    t1: &GoTermId,
    t2: &GoTermId,
) -> Option<(GoTermId, f64)> {
    let shared = dag.common_ancestors(t1, t2);
    let mut best: Option<(GoTermId, f64)> = None;
    for ancestor in shared {
        let Some(value) = ic.ic(&ancestor) else {
            continue;
        };
        match &best {
            Some((_, current)) if value <= *current => {}
            _ => best = Some((ancestor, value)),
        }
    }
    best
}

/// Pairwise IC-based similarity. Returns `None` when either term is
/// unknown, the IC source has no value for either term, or the terms have
/// no common ancestor (e.g. different namespaces).
pub fn similarity<I: InformationContent>(
    dag: &GoDag,
    ic: &I,
    t1: &GoTermId,
    t2: &GoTermId,
    method: SimilarityMethod,
) -> Option<f64> {
    let (_, ic_mica) = mica(dag, ic, t1, t2)?;
    match method {
        SimilarityMethod::Resnik => Some(ic_mica),
        SimilarityMethod::Lin => {
            let ic_a = ic.ic(t1)?;
            let ic_b = ic.ic(t2)?;
            let denom = ic_a + ic_b;
            if denom <= 0.0 {
                return Some(0.0);
            }
            Some(2.0 * ic_mica / denom)
        }
        SimilarityMethod::JiangConrath => {
            let ic_a = ic.ic(t1)?;
            let ic_b = ic.ic(t2)?;
            let dist = (ic_a + ic_b - 2.0 * ic_mica).max(0.0);
            Some(1.0 / (1.0 + dist))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct WangOptions {
    pub is_a_weight: f64,
    pub part_of_weight: f64,
}

impl Default for WangOptions {
    fn default() -> Self {
        // Wang et al. 2007 defaults.
        Self {
            is_a_weight: 0.8,
            part_of_weight: 0.6,
        }
    }
}

/// Wang 2007 structural similarity. Returns `None` if either term is
/// unknown or the two terms live in different namespaces.
pub fn wang(dag: &GoDag, t1: &GoTermId, t2: &GoTermId, opts: WangOptions) -> Option<f64> {
    let primary_a = dag.resolve(t1)?.clone();
    let primary_b = dag.resolve(t2)?.clone();
    let ns_a = dag.get(&primary_a)?.namespace;
    let ns_b = dag.get(&primary_b)?.namespace;
    if ns_a.is_none() || ns_a != ns_b {
        return None;
    }

    let s_a = wang_s_values(dag, &primary_a, opts)?;
    let s_b = wang_s_values(dag, &primary_b, opts)?;

    let sv_a: f64 = s_a.values().sum();
    let sv_b: f64 = s_b.values().sum();
    if sv_a + sv_b <= 0.0 {
        return None;
    }

    let mut numerator = 0.0;
    for (term, value_a) in &s_a {
        if let Some(value_b) = s_b.get(term) {
            numerator += value_a + value_b;
        }
    }

    Some(numerator / (sv_a + sv_b))
}

/// Compute Wang S-values for every ancestor of `term` (inclusive).
/// Walks parent edges from the seed term and propagates `weight × S(child)`
/// upward, keeping the maximum per ancestor — a longest-path computation
/// in a DAG, which terminates because S-values monotonically increase and
/// are bounded by 1.0.
fn wang_s_values(
    dag: &GoDag,
    seed: &GoTermId,
    opts: WangOptions,
) -> Option<HashMap<GoTermId, f64>> {
    dag.get(seed)?;
    let mut s: HashMap<GoTermId, f64> = HashMap::new();
    s.insert(seed.clone(), 1.0);
    let mut frontier: Vec<GoTermId> = vec![seed.clone()];
    while let Some(current) = frontier.pop() {
        let Some(s_current) = s.get(&current).copied() else {
            continue;
        };
        let Some(node) = dag.get(&current) else {
            continue;
        };
        for (parent, weight) in node
            .is_a
            .iter()
            .map(|p| (p, opts.is_a_weight))
            .chain(node.part_of.iter().map(|p| (p, opts.part_of_weight)))
        {
            if dag.get(parent).is_none() {
                continue;
            }
            let candidate = s_current * weight;
            let updated = match s.get(parent) {
                Some(existing) if *existing >= candidate => continue,
                _ => candidate,
            };
            s.insert(parent.clone(), updated);
            frontier.push(parent.clone());
        }
    }
    Some(s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SetAggregator {
    /// Average of the per-side best matches (a.k.a. funSim BMA). The
    /// recommended default for gene-vs-gene comparison.
    BestMatchAverage,
    Max,
    Average,
}

/// Aggregate pairwise similarities between two term sets. The pairwise
/// function may return `None` for individual pairs (e.g. different
/// namespaces); those are skipped. Returns `None` if no pair produces a
/// score.
pub fn set_similarity<F>(
    set_a: &[GoTermId],
    set_b: &[GoTermId],
    aggregator: SetAggregator,
    pairwise: F,
) -> Option<f64>
where
    F: FnMut(&GoTermId, &GoTermId) -> Option<f64>,
{
    if set_a.is_empty() || set_b.is_empty() {
        return None;
    }
    match aggregator {
        SetAggregator::Max => max_pairwise_similarity(set_a, set_b, pairwise),
        SetAggregator::Average => average_pairwise_similarity(set_a, set_b, pairwise),
        SetAggregator::BestMatchAverage => best_match_average_similarity(set_a, set_b, pairwise),
    }
}

fn max_pairwise_similarity<F>(
    set_a: &[GoTermId],
    set_b: &[GoTermId],
    mut pairwise: F,
) -> Option<f64>
where
    F: FnMut(&GoTermId, &GoTermId) -> Option<f64>,
{
    let mut best: Option<f64> = None;
    for a in set_a {
        for b in set_b {
            if let Some(score) = pairwise(a, b) {
                best = Some(best.map_or(score, |current| current.max(score)));
            }
        }
    }
    best
}

fn average_pairwise_similarity<F>(
    set_a: &[GoTermId],
    set_b: &[GoTermId],
    mut pairwise: F,
) -> Option<f64>
where
    F: FnMut(&GoTermId, &GoTermId) -> Option<f64>,
{
    let mut sum = 0.0;
    let mut count: u64 = 0;
    for a in set_a {
        for b in set_b {
            if let Some(score) = pairwise(a, b) {
                sum += score;
                count += 1;
            }
        }
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}

fn best_match_average_similarity<F>(
    set_a: &[GoTermId],
    set_b: &[GoTermId],
    mut pairwise: F,
) -> Option<f64>
where
    F: FnMut(&GoTermId, &GoTermId) -> Option<f64>,
{
    let avg_a = best_match_average(set_a, set_b, &mut pairwise);
    let avg_b = best_match_average(set_b, set_a, &mut |b, a| pairwise(a, b));
    match (avg_a, avg_b) {
        (Some(x), Some(y)) => Some((x + y) / 2.0),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    }
}

fn best_match_average<F>(query: &[GoTermId], target: &[GoTermId], pairwise: &mut F) -> Option<f64>
where
    F: FnMut(&GoTermId, &GoTermId) -> Option<f64>,
{
    let mut sum = 0.0;
    let mut count: u64 = 0;
    for q in query {
        let mut best: Option<f64> = None;
        for t in target {
            if let Some(score) = pairwise(q, t)
                && best.is_none_or(|current| score > current)
            {
                best = Some(score);
            }
        }
        if let Some(score) = best {
            sum += score;
            count += 1;
        }
    }
    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use genome_domain::GoNamespace;

    use super::*;
    use crate::dag::GoNode;
    use crate::ic::{CorpusIc, IntrinsicIc};

    fn term(id: &str, ns: GoNamespace, is_a: &[&str], part_of: &[&str]) -> GoNode {
        GoNode {
            id: GoTermId::new(id).unwrap(),
            namespace: Some(ns),
            is_a: is_a.iter().map(|s| GoTermId::new(*s).unwrap()).collect(),
            part_of: part_of.iter().map(|s| GoTermId::new(*s).unwrap()).collect(),
        }
    }

    fn id(s: &str) -> GoTermId {
        GoTermId::new(s).unwrap()
    }

    /// A small DAG with two sibling leaves and a more-specific leaf.
    ///   GO:0008150 (BP root)
    ///     ├─ GO:0000001 (intermediate)
    ///     │    ├─ GO:0000002 (leaf, BP)
    ///     │    └─ GO:0000003 (leaf, BP)
    ///     └─ GO:0000010 (BP, sibling of intermediate)
    fn bp_dag() -> GoDag {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term(
                "GO:0000001",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
                &[],
            ))
            .insert(term(
                "GO:0000002",
                GoNamespace::BiologicalProcess,
                &["GO:0000001"],
                &[],
            ))
            .insert(term(
                "GO:0000003",
                GoNamespace::BiologicalProcess,
                &["GO:0000001"],
                &[],
            ))
            .insert(term(
                "GO:0000010",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
                &[],
            ));
        b.build()
    }

    #[test]
    fn mica_picks_most_informative_ancestor_of_siblings() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let (ancestor, _) = mica(&dag, &ic, &id("GO:0000002"), &id("GO:0000003")).unwrap();
        assert_eq!(ancestor, id("GO:0000001"));
    }

    #[test]
    fn mica_for_distant_pair_is_root() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let (ancestor, ic_value) = mica(&dag, &ic, &id("GO:0000002"), &id("GO:0000010")).unwrap();
        assert_eq!(ancestor, id("GO:0008150"));
        assert!(ic_value.abs() < 1e-12, "root IC should be 0");
    }

    #[test]
    fn resnik_lin_jc_are_max_when_terms_are_equal() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let leaf = id("GO:0000002");
        let leaf_ic = ic.ic(&leaf).unwrap();

        let resnik = similarity(&dag, &ic, &leaf, &leaf, SimilarityMethod::Resnik).unwrap();
        assert!((resnik - leaf_ic).abs() < 1e-12);

        let lin = similarity(&dag, &ic, &leaf, &leaf, SimilarityMethod::Lin).unwrap();
        assert!((lin - 1.0).abs() < 1e-12);

        let jc = similarity(&dag, &ic, &leaf, &leaf, SimilarityMethod::JiangConrath).unwrap();
        assert!((jc - 1.0).abs() < 1e-12);
    }

    #[test]
    fn lin_for_sibling_pair_matches_closed_form() {
        // Use a corpus IC whose values are exactly computable so the
        // assertion pins down the full Lin formula. Each leaf is annotated
        // by exactly one gene; the intermediate node covers both leaf
        // genes; the distant sibling has its own gene. After true-path
        // propagation the per-namespace max count is 3 (root, all three
        // genes).
        let dag = bp_dag();
        let annotations = vec![
            vec![id("GO:0000002")],
            vec![id("GO:0000003")],
            vec![id("GO:0000010")],
        ];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);
        let leaf_ic = (3.0_f64).ln(); // -ln(1/3)
        let mid_ic = (1.5_f64).ln(); // -ln(2/3)
        let expected = 2.0 * mid_ic / (2.0 * leaf_ic);
        let actual = similarity(
            &dag,
            &ic,
            &id("GO:0000002"),
            &id("GO:0000003"),
            SimilarityMethod::Lin,
        )
        .unwrap();
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn jiang_conrath_for_sibling_pair_matches_closed_form() {
        let dag = bp_dag();
        let annotations = vec![
            vec![id("GO:0000002")],
            vec![id("GO:0000003")],
            vec![id("GO:0000010")],
        ];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);
        let leaf_ic = (3.0_f64).ln();
        let mid_ic = (1.5_f64).ln();
        // dist = 2·leaf_ic - 2·mid_ic = 2·ln(2)
        let dist = 2.0 * (leaf_ic - mid_ic);
        let expected = 1.0 / (1.0 + dist);
        let actual = similarity(
            &dag,
            &ic,
            &id("GO:0000002"),
            &id("GO:0000003"),
            SimilarityMethod::JiangConrath,
        )
        .unwrap();
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn lin_is_higher_for_closely_related_pair() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let close = similarity(
            &dag,
            &ic,
            &id("GO:0000002"),
            &id("GO:0000003"),
            SimilarityMethod::Lin,
        )
        .unwrap();
        let far = similarity(
            &dag,
            &ic,
            &id("GO:0000002"),
            &id("GO:0000010"),
            SimilarityMethod::Lin,
        )
        .unwrap();
        assert!(close > far, "close={close}, far={far}");
    }

    #[test]
    fn similarity_across_namespaces_is_none() {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term("GO:0003674", GoNamespace::MolecularFunction, &[], &[]));
        let dag = b.build();
        let ic = IntrinsicIc::from_dag(&dag);
        let value = similarity(
            &dag,
            &ic,
            &id("GO:0008150"),
            &id("GO:0003674"),
            SimilarityMethod::Resnik,
        );
        assert!(value.is_none());
    }

    #[test]
    fn max_set_similarity_returns_highest_pairwise_score() {
        let value = set_similarity(
            &[id("GO:0000001"), id("GO:0000002")],
            &[id("GO:0000003")],
            SetAggregator::Max,
            |left, _| {
                if left == &id("GO:0000001") {
                    Some(0.25)
                } else {
                    Some(0.75)
                }
            },
        )
        .unwrap();

        assert_eq!(value, 0.75);
    }

    #[test]
    fn wang_is_one_for_identical_terms() {
        let dag = bp_dag();
        let v = wang(
            &dag,
            &id("GO:0000002"),
            &id("GO:0000002"),
            WangOptions::default(),
        )
        .unwrap();
        assert!((v - 1.0).abs() < 1e-12);
    }

    #[test]
    fn wang_for_sibling_pair_matches_closed_form() {
        // bp_dag siblings: GO:0000002 and GO:0000003 each have
        //   S(self)=1, S(intermediate)=0.8, S(root)=0.8·0.8=0.64
        //   SV = 1 + 0.8 + 0.64 = 2.44
        // Common ancestors are intermediate + root:
        //   numerator = (0.8+0.8) + (0.64+0.64) = 2.88
        //   wang = 2.88 / (2.44 + 2.44) = 0.590163...
        let dag = bp_dag();
        let v = wang(
            &dag,
            &id("GO:0000002"),
            &id("GO:0000003"),
            WangOptions::default(),
        )
        .unwrap();
        let expected = 2.88 / 4.88;
        assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}");
    }

    #[test]
    fn wang_with_custom_weight_propagates_along_chain() {
        // Linear chain leaf -> mid -> root with is_a edges.
        // With weight w = 0.5: S_leaf = {leaf:1, mid:0.5, root:0.25}, SV=1.75
        // For root alone: SV = 1.
        // wang(leaf, root): common = {root}, numerator = 0.25 + 1 = 1.25
        // wang = 1.25 / 2.75
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term(
                "GO:0009987",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
                &[],
            ))
            .insert(term(
                "GO:0044238",
                GoNamespace::BiologicalProcess,
                &["GO:0009987"],
                &[],
            ));
        let dag = b.build();
        let opts = WangOptions {
            is_a_weight: 0.5,
            part_of_weight: 0.6,
        };
        let v = wang(&dag, &id("GO:0044238"), &id("GO:0008150"), opts).unwrap();
        let expected = 1.25 / 2.75;
        assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}");
    }

    #[test]
    fn wang_keeps_maximum_when_a_smaller_candidate_arrives_later() {
        // Asymmetric DAG with two paths from X to R: a 2-step path via C
        // and a 3-step path via A→B. The traversal pops C first (LIFO,
        // part_of iterated after is_a), so R is set to the LARGER value
        // first; the 3-step path then offers a SMALLER candidate later.
        // The `*existing >= candidate` guard must drop that smaller
        // update — replacing it with `false` would always overwrite and
        // change the answer.
        //
        //   X (GO:0000023)
        //     ├─ is_a    → A (GO:0000024) ─is_a→ B (GO:0000026) ─part_of→ R
        //     │            contribution: 1 · 0.8 · 0.8 · 0.6 = 0.384
        //     └─ part_of → C (GO:0000025) ─is_a→ R
        //                  contribution: 1 · 0.6 · 0.8 = 0.48
        //
        // S(R) must be max(0.48, 0.384) = 0.48.
        let mut b = GoDag::builder();
        b.insert(term("GO:0000020", GoNamespace::BiologicalProcess, &[], &[])) // R
            .insert(term(
                "GO:0000026",
                GoNamespace::BiologicalProcess,
                &[],
                &["GO:0000020"],
            )) // B part_of R
            .insert(term(
                "GO:0000024",
                GoNamespace::BiologicalProcess,
                &["GO:0000026"],
                &[],
            )) // A is_a B
            .insert(term(
                "GO:0000025",
                GoNamespace::BiologicalProcess,
                &["GO:0000020"],
                &[],
            )) // C is_a R
            .insert(term(
                "GO:0000023",
                GoNamespace::BiologicalProcess,
                &["GO:0000024"],
                &["GO:0000025"],
            )); // X is_a A, part_of C
        let dag = b.build();

        let v = wang(
            &dag,
            &id("GO:0000023"),
            &id("GO:0000020"),
            WangOptions::default(),
        )
        .unwrap();

        // Subgraph(X) = {X:1, A:0.8, B:0.64, C:0.6, R:0.48}.
        let sv_x = 1.0 + 0.8 + 0.64 + 0.6 + 0.48;
        let sv_r = 1.0;
        let numerator = 0.48 + 1.0;
        let expected = numerator / (sv_x + sv_r);
        assert!((v - expected).abs() < 1e-9, "expected {expected}, got {v}");
    }

    #[test]
    fn wang_is_higher_for_sibling_than_distant_pair() {
        let dag = bp_dag();
        let close = wang(
            &dag,
            &id("GO:0000002"),
            &id("GO:0000003"),
            WangOptions::default(),
        )
        .unwrap();
        let far = wang(
            &dag,
            &id("GO:0000002"),
            &id("GO:0000010"),
            WangOptions::default(),
        )
        .unwrap();
        assert!(close > far, "close={close}, far={far}");
        assert!(close > 0.0 && close <= 1.0);
        assert!(far > 0.0 && far <= 1.0);
    }

    #[test]
    fn wang_handles_part_of_edges() {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term(
                "GO:0044238",
                GoNamespace::BiologicalProcess,
                &[],
                &["GO:0008150"],
            ));
        let dag = b.build();
        let v = wang(
            &dag,
            &id("GO:0044238"),
            &id("GO:0008150"),
            WangOptions::default(),
        )
        .unwrap();
        // Different terms → sim < 1, but a positive value (they share the root).
        assert!(v > 0.0 && v < 1.0, "got {v}");
    }

    #[test]
    fn wang_across_namespaces_is_none() {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term("GO:0003674", GoNamespace::MolecularFunction, &[], &[]));
        let dag = b.build();
        let v = wang(
            &dag,
            &id("GO:0008150"),
            &id("GO:0003674"),
            WangOptions::default(),
        );
        assert!(v.is_none());
    }

    #[test]
    fn set_similarity_bma_average_max_agree_on_identical_singleton_sets() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let set = vec![id("GO:0000002")];
        let pairwise =
            |a: &GoTermId, b: &GoTermId| similarity(&dag, &ic, a, b, SimilarityMethod::Lin);
        for agg in [
            SetAggregator::Max,
            SetAggregator::Average,
            SetAggregator::BestMatchAverage,
        ] {
            let v = set_similarity(&set, &set, agg, pairwise).unwrap();
            assert!((v - 1.0).abs() < 1e-12, "{agg:?} → {v}");
        }
    }

    #[test]
    fn set_similarity_max_picks_highest_score_over_unequal_pairs() {
        // Two-element set_a, singleton set_b. The LOWER-scoring pair is
        // visited first so the per-iteration `score > current` branch is
        // exercised on an ascending pair — replacing `>` with `<` or `==`
        // would lock the answer at the smaller score instead of updating
        // to the larger one.
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let v = set_similarity(
            // root first → Lin(root, leaf) = 0; leaf second → Lin(leaf, leaf) = 1.
            &[id("GO:0008150"), id("GO:0000002")],
            &[id("GO:0000002")],
            SetAggregator::Max,
            |a, b| similarity(&dag, &ic, a, b, SimilarityMethod::Lin),
        )
        .unwrap();
        assert!((v - 1.0).abs() < 1e-12, "got {v}");
    }

    #[test]
    fn set_similarity_average_divides_by_pair_count() {
        // Two pairs with distinct, non-zero Lin scores. Average must equal
        // (s1 + s2) / 2 — replacing `/` with `*` would yield 2·(s1 + s2),
        // dropping the / from the formula.
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let pairwise =
            |a: &GoTermId, b: &GoTermId| similarity(&dag, &ic, a, b, SimilarityMethod::Lin);
        let self_score = pairwise(&id("GO:0000002"), &id("GO:0000002")).unwrap();
        let sib_score = pairwise(&id("GO:0000002"), &id("GO:0000003")).unwrap();
        let v = set_similarity(
            &[id("GO:0000002")],
            &[id("GO:0000002"), id("GO:0000003")],
            SetAggregator::Average,
            pairwise,
        )
        .unwrap();
        let expected = (self_score + sib_score) / 2.0;
        assert!((v - expected).abs() < 1e-12, "expected {expected}, got {v}");
        // Sanity: the two scores are different, so / vs * is detectable.
        assert!((self_score - sib_score).abs() > 1e-6);
    }

    #[test]
    fn set_similarity_bma_updates_best_when_higher_score_comes_after_lower() {
        // For a single query against a 2-target set, ensure the LOWER
        // score is visited first so the `score > current` update branch
        // in best_match_average actually fires on an ascending input —
        // killing `>` → `==` mutations there.
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let pairwise =
            |a: &GoTermId, b: &GoTermId| similarity(&dag, &ic, a, b, SimilarityMethod::Lin);
        // q = distant_sibling; targets ordered as [leaf1 (Lin=0), distant_self (Lin=1)].
        let actual = set_similarity(
            &[id("GO:0000010")],
            &[id("GO:0000002"), id("GO:0000010")],
            SetAggregator::BestMatchAverage,
            pairwise,
        )
        .unwrap();
        // a-side: best is Lin(distant, distant) = 1.0 → avg = 1.0.
        // b-side:
        //   q=leaf1, t=distant: 0.0 → best=0.0
        //   q=distant, t=distant: 1.0 → best=1.0
        //   avg = 0.5
        // BMA = (1.0 + 0.5) / 2 = 0.75
        let expected = 0.75;
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn set_similarity_bma_averages_per_side_best_matches() {
        // set_a = {leaf1, distant_sibling}; set_b = {leaf1, leaf2_sib}.
        // The per-side averages differ from each other and from any
        // individual pair, so a `/` → `*` mutation on either side's
        // average changes the answer.
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let pairwise =
            |a: &GoTermId, b: &GoTermId| similarity(&dag, &ic, a, b, SimilarityMethod::Lin);
        let set_a = vec![id("GO:0000002"), id("GO:0000010")];
        let set_b = vec![id("GO:0000002"), id("GO:0000003")];
        let actual =
            set_similarity(&set_a, &set_b, SetAggregator::BestMatchAverage, pairwise).unwrap();

        let lin = |t1: &GoTermId, t2: &GoTermId| pairwise(t1, t2).unwrap_or(0.0);
        let a_best0 = lin(&set_a[0], &set_b[0]).max(lin(&set_a[0], &set_b[1]));
        let a_best1 = lin(&set_a[1], &set_b[0]).max(lin(&set_a[1], &set_b[1]));
        let b_best0 = lin(&set_b[0], &set_a[0]).max(lin(&set_b[0], &set_a[1]));
        let b_best1 = lin(&set_b[1], &set_a[0]).max(lin(&set_b[1], &set_a[1]));
        let expected = ((a_best0 + a_best1) / 2.0 + (b_best0 + b_best1) / 2.0) / 2.0;
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
        // Sanity: actual ∈ (0, 1) strict — kills constant-return mutants.
        assert!(actual > 0.0 && actual < 1.0);
    }

    #[test]
    fn set_similarity_returns_none_for_empty_set() {
        let dag = bp_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let empty: Vec<GoTermId> = Vec::new();
        let other = vec![id("GO:0000002")];
        let v = set_similarity(&empty, &other, SetAggregator::BestMatchAverage, |a, b| {
            similarity(&dag, &ic, a, b, SimilarityMethod::Lin)
        });
        assert!(v.is_none());
    }

    #[test]
    fn set_similarity_skips_unresolvable_pairs() {
        // Two singleton sets, one term in each, in different namespaces:
        // every pairwise call returns None → set similarity also None.
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term("GO:0003674", GoNamespace::MolecularFunction, &[], &[]));
        let dag = b.build();
        let ic = IntrinsicIc::from_dag(&dag);
        let v = set_similarity(
            &[id("GO:0008150")],
            &[id("GO:0003674")],
            SetAggregator::BestMatchAverage,
            |a, b| similarity(&dag, &ic, a, b, SimilarityMethod::Lin),
        );
        assert!(v.is_none());
    }

    #[test]
    fn resnik_uses_corpus_ic_distinct_from_intrinsic() {
        // Sanity: similarity is computable from a CorpusIc as well.
        let dag = bp_dag();
        let annotations = vec![
            vec![id("GO:0000002")],
            vec![id("GO:0000003")],
            vec![id("GO:0000010")],
        ];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);
        let v = similarity(
            &dag,
            &ic,
            &id("GO:0000002"),
            &id("GO:0000003"),
            SimilarityMethod::Resnik,
        )
        .unwrap();
        // GO:0000001 covers 2 of 3 annotated genes (-ln(2/3) ≈ 0.405).
        assert!((v - (1.5_f64).ln()).abs() < 1e-9, "got {v}");
    }
}
