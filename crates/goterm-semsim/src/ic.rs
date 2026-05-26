//! Information content for GO terms.
//!
//! Two complementary IC sources are provided:
//!
//! * [`IntrinsicIc`] — Seco et al. 2004. IC derived from the DAG topology
//!   alone, so no corpus is required:
//!
//!   `IC(t) = 1 - log(|descendants(t) ∪ {t}|) / log(N_namespace)`
//!
//!   `N_namespace` is the number of terms in `t`'s namespace, so each
//!   namespace's IC is bounded between 0 (root) and 1 (leaf).
//!
//! * [`CorpusIc`] — frequentist IC from annotation counts:
//!
//!   `IC(t) = -log(count(t) / N_namespace)`
//!
//!   `count(t)` is the number of unique items (e.g. genes) annotated with
//!   `t` *or any of its descendants* — the "true path rule" expansion is
//!   handled inside [`CorpusIc::from_gene_annotations`]. `N_namespace` is
//!   the maximum per-term count in that namespace (the namespace root in
//!   well-formed GO data).

use std::collections::{HashMap, HashSet};

use genome_core::{GoNamespace, GoTermId};

use crate::dag::GoDag;

/// Shared interface so similarity functions can be parameterised over any
/// IC source.
pub trait InformationContent {
    fn ic(&self, term: &GoTermId) -> Option<f64>;
}

#[derive(Debug, Clone, Default)]
pub struct IntrinsicIc {
    values: HashMap<GoTermId, f64>,
}

impl IntrinsicIc {
    pub fn from_dag(dag: &GoDag) -> Self {
        let mut per_namespace: HashMap<GoNamespace, usize> = HashMap::new();
        for node in dag.iter() {
            if let Some(ns) = node.namespace {
                *per_namespace.entry(ns).or_default() += 1;
            }
        }

        let mut values: HashMap<GoTermId, f64> = HashMap::new();
        for node in dag.iter() {
            let Some(ns) = node.namespace else {
                continue;
            };
            let n = per_namespace.get(&ns).copied().unwrap_or(0);
            if n <= 1 {
                values.insert(node.id.clone(), 0.0);
                continue;
            }
            let descendants = dag
                .descendants(&node.id)
                .map(|s| s.len())
                .unwrap_or(1)
                .max(1);
            let ic = 1.0 - (descendants as f64).ln() / (n as f64).ln();
            values.insert(node.id.clone(), ic.clamp(0.0, 1.0));
        }

        Self { values }
    }
}

impl InformationContent for IntrinsicIc {
    fn ic(&self, term: &GoTermId) -> Option<f64> {
        self.values.get(term).copied()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CorpusIc {
    values: HashMap<GoTermId, f64>,
}

impl CorpusIc {
    /// Build corpus IC from per-item annotation lists.
    ///
    /// Each element of `annotations` is the set of GO term ids assigned to
    /// one item (typically one gene). Terms are expanded via the DAG's
    /// ancestor closure ("true path rule") and unique items are counted
    /// per term. Annotations to terms that are not in `dag` are dropped.
    pub fn from_gene_annotations<I, J>(dag: &GoDag, annotations: I) -> Self
    where
        I: IntoIterator<Item = J>,
        J: IntoIterator<Item = GoTermId>,
    {
        let mut counts: HashMap<GoTermId, u64> = HashMap::new();
        for item_terms in annotations {
            let mut expanded: HashSet<GoTermId> = HashSet::new();
            for term in item_terms {
                let Some(primary) = dag.resolve(&term) else {
                    continue;
                };
                if let Some(anc) = dag.ancestors(primary) {
                    expanded.extend(anc.iter().cloned());
                }
            }
            for term in expanded {
                *counts.entry(term).or_default() += 1;
            }
        }
        Self::from_propagated_counts(dag, counts)
    }

    /// Build corpus IC directly from already-propagated counts. Useful
    /// when caller has pre-computed per-term counts (e.g. from a database
    /// summary table) and doesn't want to re-do the ancestor expansion.
    pub fn from_propagated_counts(dag: &GoDag, counts: HashMap<GoTermId, u64>) -> Self {
        let mut max_per_namespace: HashMap<GoNamespace, u64> = HashMap::new();
        for (term, count) in &counts {
            let Some(node) = dag.get(term) else { continue };
            let Some(ns) = node.namespace else { continue };
            let slot = max_per_namespace.entry(ns).or_default();
            if *count > *slot {
                *slot = *count;
            }
        }

        let mut values: HashMap<GoTermId, f64> = HashMap::new();
        for (term, count) in counts {
            if count == 0 {
                continue;
            }
            let Some(node) = dag.get(&term) else { continue };
            let Some(ns) = node.namespace else { continue };
            let total = max_per_namespace.get(&ns).copied().unwrap_or(0);
            if total == 0 {
                continue;
            }
            let p = (count as f64) / (total as f64);
            // p ∈ (0, 1]; ln(1) = 0 is the correct IC of the root.
            let ic = -p.ln();
            values.insert(term, ic.max(0.0));
        }
        Self { values }
    }
}

impl InformationContent for CorpusIc {
    fn ic(&self, term: &GoTermId) -> Option<f64> {
        self.values.get(term).copied()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::dag::GoNode;

    fn term(id: &str, ns: GoNamespace, is_a: &[&str]) -> GoNode {
        GoNode {
            id: GoTermId::new(id).unwrap(),
            namespace: Some(ns),
            is_a: is_a.iter().map(|s| GoTermId::new(*s).unwrap()).collect(),
            part_of: Vec::new(),
        }
    }

    fn id(s: &str) -> GoTermId {
        GoTermId::new(s).unwrap()
    }

    /// BP namespace with one root and two leaves:
    ///   root -> a -> leaf1
    ///        \-> leaf2
    fn small_dag() -> GoDag {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[]))
            .insert(term(
                "GO:0000001",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
            ))
            .insert(term(
                "GO:0000002",
                GoNamespace::BiologicalProcess,
                &["GO:0000001"],
            ))
            .insert(term(
                "GO:0000003",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
            ));
        b.build()
    }

    #[test]
    fn intrinsic_ic_root_is_zero_leaf_is_one() {
        let dag = small_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let root_ic = ic.ic(&id("GO:0008150")).unwrap();
        let leaf_ic = ic.ic(&id("GO:0000002")).unwrap();
        assert!(root_ic.abs() < 1e-12, "root IC should be 0, got {root_ic}");
        assert!(
            (leaf_ic - 1.0).abs() < 1e-12,
            "leaf IC should be 1, got {leaf_ic}"
        );
    }

    #[test]
    fn intrinsic_ic_intermediate_is_between_root_and_leaf() {
        let dag = small_dag();
        let ic = IntrinsicIc::from_dag(&dag);
        let mid = ic.ic(&id("GO:0000001")).unwrap();
        assert!(
            mid > 0.0 && mid < 1.0,
            "intermediate IC out of range: {mid}"
        );
    }

    #[test]
    fn corpus_ic_more_specific_term_has_higher_ic() {
        let dag = small_dag();
        // gene1 annotated with leaf1; gene2 annotated with leaf2 sibling;
        // both implicitly annotate root and any intermediate ancestors.
        let annotations = vec![vec![id("GO:0000002")], vec![id("GO:0000003")]];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);

        let root_ic = ic.ic(&id("GO:0008150")).unwrap();
        let leaf_ic = ic.ic(&id("GO:0000002")).unwrap();
        // Root is annotated by both genes (count = max = 2), so IC(root) = 0.
        assert!(root_ic.abs() < 1e-12);
        // Leaf is annotated by 1 gene out of 2 → IC = -ln(0.5) ≈ 0.693.
        assert!((leaf_ic - (2.0_f64).ln()).abs() < 1e-9);
    }

    #[test]
    fn corpus_ic_ignores_unknown_terms() {
        let dag = small_dag();
        let annotations = vec![vec![id("GO:9999999"), id("GO:0000002")]];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);
        // Unknown term contributes nothing.
        assert!(ic.ic(&id("GO:9999999")).is_none());
        // Known term still gets a value.
        assert!(ic.ic(&id("GO:0000002")).is_some());
    }

    #[test]
    fn corpus_ic_resolves_alt_ids_through_dag() {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[]))
            .insert(term(
                "GO:0000001",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
            ))
            .alias(id("GO:0000099"), id("GO:0000001"));
        let dag = b.build();
        let annotations = vec![vec![id("GO:0000099")]];
        let ic = CorpusIc::from_gene_annotations(&dag, annotations);
        // The alt id should resolve to GO:0000001 and propagate to the root.
        assert!(ic.ic(&id("GO:0000001")).is_some());
        assert!(ic.ic(&id("GO:0008150")).is_some());
    }

    #[test]
    fn intrinsic_ic_handles_single_term_namespace() {
        let mut b = GoDag::builder();
        b.insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[]));
        let dag = b.build();
        let ic = IntrinsicIc::from_dag(&dag);
        // Only term in the namespace → log(N) = 0; we fall back to 0.
        assert_eq!(ic.ic(&id("GO:0008150")), Some(0.0));
    }
}
