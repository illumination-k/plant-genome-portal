//! Directed acyclic graph of Gene Ontology terms.
//!
//! `GoDag` is a pure in-memory view of GO terms keyed by primary
//! [`GoTermId`], with `is_a` and `part_of` parent edges and a precomputed
//! ancestor / descendant closure (over both edge kinds, matching the GO
//! "true path rule"). It exists as the substrate every semantic-similarity
//! algorithm in this crate operates on; callers build it once from any
//! source (the OBO loader in genome-store, a hand-written test fixture, etc.)
//! via [`GoDag::builder`].

use std::collections::{HashMap, HashSet, VecDeque};

use genome_domain::{GoNamespace, GoTermId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoNode {
    pub id: GoTermId,
    pub namespace: Option<GoNamespace>,
    pub is_a: Vec<GoTermId>,
    pub part_of: Vec<GoTermId>,
}

impl GoNode {
    /// Iterate over parents via both `is_a` and `part_of`.
    pub fn structural_parents(&self) -> impl Iterator<Item = &GoTermId> {
        self.is_a.iter().chain(self.part_of.iter())
    }
}

#[derive(Debug, Clone, Default)]
pub struct GoDag {
    nodes: HashMap<GoTermId, GoNode>,
    alt_to_primary: HashMap<GoTermId, GoTermId>,
    /// For each primary term, the inclusive ancestor closure (term + all
    /// ancestors via `is_a` ∪ `part_of`). Inclusive so MICA lookups and
    /// IC propagation can treat a term as its own ancestor uniformly.
    ancestors: HashMap<GoTermId, HashSet<GoTermId>>,
    /// For each primary term, the inclusive descendant closure (used by
    /// intrinsic IC).
    descendants: HashMap<GoTermId, HashSet<GoTermId>>,
}

impl GoDag {
    pub fn builder() -> GoDagBuilder {
        GoDagBuilder::default()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get(&self, id: &GoTermId) -> Option<&GoNode> {
        self.nodes.get(id)
    }

    /// Resolve `id` (which may be a primary id or an `alt_id`) to its
    /// primary identifier. Returns `None` if the id is unknown.
    pub fn resolve(&self, id: &GoTermId) -> Option<&GoTermId> {
        if let Some((primary, _)) = self.nodes.get_key_value(id) {
            return Some(primary);
        }
        self.alt_to_primary.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &GoNode> {
        self.nodes.values()
    }

    /// Inclusive ancestor closure (the term itself + every ancestor via
    /// `is_a` ∪ `part_of`). Returns `None` if the term is unknown.
    pub fn ancestors(&self, id: &GoTermId) -> Option<&HashSet<GoTermId>> {
        let primary = self.resolve(id)?;
        self.ancestors.get(primary)
    }

    /// Inclusive descendant closure (the term itself + every descendant).
    pub fn descendants(&self, id: &GoTermId) -> Option<&HashSet<GoTermId>> {
        let primary = self.resolve(id)?;
        self.descendants.get(primary)
    }

    /// Common ancestors of `a` and `b` (intersection of their inclusive
    /// ancestor closures). Empty when the two terms live in different
    /// namespaces or one is unknown.
    pub fn common_ancestors(&self, a: &GoTermId, b: &GoTermId) -> HashSet<GoTermId> {
        let (Some(aa), Some(bb)) = (self.ancestors(a), self.ancestors(b)) else {
            return HashSet::new();
        };
        aa.intersection(bb).cloned().collect()
    }
}

#[derive(Debug, Default, Clone)]
pub struct GoDagBuilder {
    nodes: HashMap<GoTermId, GoNode>,
    alt_to_primary: HashMap<GoTermId, GoTermId>,
}

impl GoDagBuilder {
    /// Insert a term. Later inserts with the same primary id overwrite
    /// the previous entry.
    pub fn insert(&mut self, node: GoNode) -> &mut Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Register an `alt_id` aliasing to a primary id. The primary does
    /// not need to exist at the time of this call.
    pub fn alias(&mut self, alt_id: GoTermId, primary: GoTermId) -> &mut Self {
        self.alt_to_primary.insert(alt_id, primary);
        self
    }

    pub fn build(self) -> GoDag {
        let mut ancestors: HashMap<GoTermId, HashSet<GoTermId>> = HashMap::new();
        let mut descendants: HashMap<GoTermId, HashSet<GoTermId>> = HashMap::new();
        for id in self.nodes.keys() {
            let anc = bfs_closure(id, &self.nodes, |node| node.structural_parents());
            descendants
                .entry(id.clone())
                .or_default()
                .insert(id.clone());
            for ancestor in &anc {
                descendants
                    .entry(ancestor.clone())
                    .or_default()
                    .insert(id.clone());
            }
            let mut anc_inclusive = anc;
            anc_inclusive.insert(id.clone());
            ancestors.insert(id.clone(), anc_inclusive);
        }
        // Terms with no descendants still need an entry (just themselves).
        for id in self.nodes.keys() {
            descendants
                .entry(id.clone())
                .or_default()
                .insert(id.clone());
        }
        GoDag {
            nodes: self.nodes,
            alt_to_primary: self.alt_to_primary,
            ancestors,
            descendants,
        }
    }
}

/// BFS from `start` following edges produced by `edges_of`. Returns the
/// set of reachable nodes *excluding* `start` itself.
fn bfs_closure<'a, F, I>(
    start: &GoTermId,
    nodes: &'a HashMap<GoTermId, GoNode>,
    mut edges_of: F,
) -> HashSet<GoTermId>
where
    F: FnMut(&'a GoNode) -> I,
    I: IntoIterator<Item = &'a GoTermId>,
{
    let mut visited: HashSet<GoTermId> = HashSet::new();
    let mut queue: VecDeque<&GoTermId> = VecDeque::new();
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        let Some(node) = nodes.get(current) else {
            continue;
        };
        for next in edges_of(node) {
            if next == start {
                continue;
            }
            if visited.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }
    visited
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

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

    /// Three-level chain in BP:
    ///   GO:0008150 (BP root)
    ///     └─ GO:0009987 (cellular process)
    ///           └─ GO:0044238 (primary metabolic process)
    fn linear_dag() -> GoDag {
        let mut builder = GoDag::builder();
        builder
            .insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
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
        builder.build()
    }

    #[test]
    fn len_and_is_empty_match_node_count() {
        let dag = linear_dag();
        assert_eq!(dag.len(), 3);
        assert!(!dag.is_empty());

        let empty = GoDag::builder().build();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn ancestors_are_inclusive_and_follow_is_a() {
        let dag = linear_dag();
        let anc = dag.ancestors(&id("GO:0044238")).unwrap();
        assert!(anc.contains(&id("GO:0044238")));
        assert!(anc.contains(&id("GO:0009987")));
        assert!(anc.contains(&id("GO:0008150")));
        assert_eq!(anc.len(), 3);

        // Root only sees itself.
        let root_anc = dag.ancestors(&id("GO:0008150")).unwrap();
        assert_eq!(root_anc.len(), 1);
        assert!(root_anc.contains(&id("GO:0008150")));
    }

    #[test]
    fn descendants_are_inclusive_and_inverted() {
        let dag = linear_dag();
        let root_desc = dag.descendants(&id("GO:0008150")).unwrap();
        assert_eq!(root_desc.len(), 3);
        assert!(root_desc.contains(&id("GO:0044238")));

        let leaf_desc = dag.descendants(&id("GO:0044238")).unwrap();
        assert_eq!(leaf_desc.len(), 1);
        assert!(leaf_desc.contains(&id("GO:0044238")));
    }

    #[test]
    fn part_of_edges_contribute_to_ancestors() {
        let mut builder = GoDag::builder();
        builder
            .insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .insert(term(
                "GO:0044238",
                GoNamespace::BiologicalProcess,
                &[],
                &["GO:0008150"],
            ));
        let dag = builder.build();
        let anc = dag.ancestors(&id("GO:0044238")).unwrap();
        assert!(anc.contains(&id("GO:0008150")));
    }

    #[test]
    fn common_ancestors_intersects_inclusive_closures() {
        let mut builder = GoDag::builder();
        builder
            .insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
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
            ))
            .insert(term(
                "GO:0050896",
                GoNamespace::BiologicalProcess,
                &["GO:0008150"],
                &[],
            ));
        let dag = builder.build();

        let shared = dag.common_ancestors(&id("GO:0044238"), &id("GO:0050896"));
        assert!(shared.contains(&id("GO:0008150")));
        assert!(!shared.contains(&id("GO:0009987")));
    }

    #[test]
    fn common_ancestors_empty_for_unknown_term() {
        let dag = linear_dag();
        let shared = dag.common_ancestors(&id("GO:0044238"), &id("GO:9999999"));
        assert!(shared.is_empty());
    }

    #[test]
    fn resolve_returns_primary_for_alt_id() {
        let mut builder = GoDag::builder();
        builder
            .insert(term("GO:0008150", GoNamespace::BiologicalProcess, &[], &[]))
            .alias(id("GO:0000004"), id("GO:0008150"));
        let dag = builder.build();
        assert_eq!(dag.resolve(&id("GO:0000004")), Some(&id("GO:0008150")));
        assert_eq!(dag.resolve(&id("GO:0008150")), Some(&id("GO:0008150")));
        assert!(dag.resolve(&id("GO:9999999")).is_none());
    }

    #[test]
    fn ancestors_handles_cycle_without_looping() {
        // Synthetic cycle: A -> B -> A. The closure must terminate.
        let mut builder = GoDag::builder();
        builder
            .insert(term(
                "GO:0000001",
                GoNamespace::BiologicalProcess,
                &["GO:0000002"],
                &[],
            ))
            .insert(term(
                "GO:0000002",
                GoNamespace::BiologicalProcess,
                &["GO:0000001"],
                &[],
            ));
        let dag = builder.build();
        let anc = dag.ancestors(&id("GO:0000001")).unwrap();
        // Self + the other node; cycle visit doesn't add anything else.
        assert_eq!(anc.len(), 2);
        assert!(anc.contains(&id("GO:0000001")));
        assert!(anc.contains(&id("GO:0000002")));
    }
}
