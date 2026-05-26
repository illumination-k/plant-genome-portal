//! Graph view of a co-expression network.
//!
//! Wraps `petgraph` so callers can threshold a [`CoexpressionIndex`] into an
//! undirected graph and run clustering / neighborhood queries on top of it.
//! Connected components on a thresholded graph are the standard first-pass
//! co-expression module definition (cf. ATTED-II HRR cutoffs).

use std::collections::{HashMap, HashSet};

use genome_core::GeneId;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::{Bfs, EdgeRef};
use rayon::prelude::*;

use crate::{CoexpressionEdge, CoexpressionIndex};

/// Predicate that selects gene pairs from a [`CoexpressionIndex`].
///
/// Each variant uses the symmetric metric of the same name. `MutualRankAtMost`
/// and `HighestReciprocalRankAtMost` are the conventional thresholds for
/// hard-cutoff co-expression networks; `LogitScoreAtLeast` keeps highly ranked
/// pairs regardless of network size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeFilter {
    HighestReciprocalRankAtMost(usize),
    MutualRankAtMost(f64),
    LogitScoreAtLeast(f64),
}

impl EdgeFilter {
    fn matches(&self, index: &CoexpressionIndex, source: usize, target: usize) -> bool {
        match *self {
            Self::HighestReciprocalRankAtMost(threshold) => index
                .highest_reciprocal_rank_by_index(source, target)
                .is_some_and(|hrr| hrr <= threshold),
            Self::MutualRankAtMost(threshold) => index
                .mutual_rank_by_index(source, target)
                .is_some_and(|mr| mr <= threshold),
            Self::LogitScoreAtLeast(threshold) => index
                .logit_score_by_index(source, target)
                .is_some_and(|ls| ls >= threshold),
        }
    }
}

/// Undirected co-expression graph keyed by `GeneId`.
#[derive(Debug, Clone)]
pub struct CoexpressionGraph {
    inner: UnGraph<GeneId, CoexpressionEdge>,
    gene_to_node: HashMap<GeneId, NodeIndex>,
}

impl CoexpressionGraph {
    /// Builds an undirected graph from a co-expression index, keeping only
    /// gene pairs that satisfy `filter`. All genes are added as nodes so
    /// isolated genes show up as singleton components.
    pub fn from_index(index: &CoexpressionIndex, filter: EdgeFilter) -> Self {
        let gene_count = index.gene_count();
        let mut graph = UnGraph::<GeneId, CoexpressionEdge>::with_capacity(gene_count, 0);
        let mut gene_to_node = HashMap::with_capacity(gene_count);
        for gene in &index.gene_ids {
            let node = graph.add_node(gene.clone());
            gene_to_node.insert(gene.clone(), node);
        }

        let pairs: Vec<(usize, usize)> = (0..gene_count)
            .into_par_iter()
            .flat_map_iter(|source| {
                (source + 1..gene_count)
                    .filter(move |&target| filter.matches(index, source, target))
                    .map(move |target| (source, target))
            })
            .collect();

        for (source, target) in pairs {
            if let Some(edge) = index.edge_by_index(source, target) {
                graph.add_edge(NodeIndex::new(source), NodeIndex::new(target), edge);
            }
        }

        Self {
            inner: graph,
            gene_to_node,
        }
    }

    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    pub fn contains_gene(&self, gene: &GeneId) -> bool {
        self.gene_to_node.contains_key(gene)
    }

    /// Edges incident to `gene`, paired with the partner gene id.
    pub fn neighbors(&self, gene: &GeneId) -> Vec<(GeneId, CoexpressionEdge)> {
        let Some(&node) = self.gene_to_node.get(gene) else {
            return Vec::new();
        };
        self.inner
            .edges(node)
            .map(|edge_ref| {
                let other = if edge_ref.source() == node {
                    edge_ref.target()
                } else {
                    edge_ref.source()
                };
                (self.inner[other].clone(), edge_ref.weight().clone())
            })
            .collect()
    }

    /// Returns one cluster per connected component. Order is deterministic
    /// (matches insertion order from [`Self::from_index`]).
    pub fn connected_components(&self) -> Vec<Vec<GeneId>> {
        let mut visited: HashSet<NodeIndex> = HashSet::with_capacity(self.inner.node_count());
        let mut components = Vec::new();
        for start in self.inner.node_indices() {
            if visited.contains(&start) {
                continue;
            }
            let mut component = Vec::new();
            let mut bfs = Bfs::new(&self.inner, start);
            while let Some(node) = bfs.next(&self.inner) {
                visited.insert(node);
                component.push(self.inner[node].clone());
            }
            components.push(component);
        }
        components
    }

    /// Extracts the subgraph containing every gene reachable from `seeds`
    /// within `hops` edge traversals. Seeds that are not in the graph are
    /// silently skipped.
    pub fn subgraph_around(&self, seeds: &[GeneId], hops: usize) -> Self {
        let included = self.collect_within_hops(seeds, hops);
        let mut graph = UnGraph::<GeneId, CoexpressionEdge>::with_capacity(included.len(), 0);
        let mut gene_to_node = HashMap::with_capacity(included.len());
        let mut old_to_new: HashMap<NodeIndex, NodeIndex> = HashMap::with_capacity(included.len());
        for &old in &included {
            let gene = self.inner[old].clone();
            let new = graph.add_node(gene.clone());
            gene_to_node.insert(gene, new);
            old_to_new.insert(old, new);
        }
        for edge_ref in self.inner.edge_references() {
            if let (Some(&source), Some(&target)) = (
                old_to_new.get(&edge_ref.source()),
                old_to_new.get(&edge_ref.target()),
            ) {
                graph.add_edge(source, target, edge_ref.weight().clone());
            }
        }
        Self {
            inner: graph,
            gene_to_node,
        }
    }

    /// Borrow the underlying `petgraph` graph for callers that want to compose
    /// additional algorithms.
    pub fn as_petgraph(&self) -> &UnGraph<GeneId, CoexpressionEdge> {
        &self.inner
    }

    fn collect_within_hops(&self, seeds: &[GeneId], hops: usize) -> Vec<NodeIndex> {
        let mut included = Vec::new();
        let mut seen: HashSet<NodeIndex> = HashSet::new();
        let mut frontier = Vec::new();
        for gene in seeds {
            if let Some(&node) = self.gene_to_node.get(gene)
                && seen.insert(node)
            {
                included.push(node);
                frontier.push(node);
            }
        }
        for _ in 0..hops {
            let mut next = Vec::new();
            for &node in &frontier {
                for neighbor in self.inner.neighbors(node) {
                    if seen.insert(neighbor) {
                        included.push(neighbor);
                        next.push(neighbor);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        included
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::{CoexpressionOptions, build_coexpression_index};
    use expression_core::{ExpressionMatrix, ExpressionUnit, SraRunAccession};
    use genome_core::AssemblyAccession;

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new("GCA_037833805.1").unwrap()
    }

    fn gene(id: &str) -> GeneId {
        GeneId::new(id).unwrap()
    }

    fn runs(count: usize) -> Vec<SraRunAccession> {
        (1..=count)
            .map(|index| SraRunAccession::new(format!("SRR{index:06}")).unwrap())
            .collect()
    }

    /// Two tight modules ({g1,g2,g3} fully correlated, {g4,g5} fully
    /// correlated) plus an isolated gene g6 with constant expression.
    fn modular_index() -> CoexpressionIndex {
        let matrix = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![
                gene("g1"),
                gene("g2"),
                gene("g3"),
                gene("g4"),
                gene("g5"),
                gene("g6"),
            ],
            runs(4),
            vec![
                1.0, 2.0, 3.0, 4.0, // g1
                2.0, 4.0, 6.0, 8.0, // g2 (perfectly correlated with g1)
                3.0, 6.0, 9.0, 12.0, // g3 (perfectly correlated with g1)
                10.0, 5.0, 7.0, 2.0, // g4
                20.0, 10.0, 14.0, 4.0, // g5 (perfectly correlated with g4)
                1.0, 1.0, 1.0, 1.0, // g6 (constant -> NaN correlations)
            ],
        )
        .unwrap();
        build_coexpression_index(&matrix, CoexpressionOptions::default()).unwrap()
    }

    #[test]
    fn hrr_threshold_recovers_two_modules_plus_isolated_gene() {
        let index = modular_index();
        let graph =
            CoexpressionGraph::from_index(&index, EdgeFilter::HighestReciprocalRankAtMost(2));

        assert_eq!(graph.node_count(), 6);
        let mut components = graph.connected_components();
        components.sort_by_key(|component| component[0].as_str().to_string());

        let module_one: Vec<&str> = components[0].iter().map(GeneId::as_str).collect();
        let module_two: Vec<&str> = components[1].iter().map(GeneId::as_str).collect();
        let isolated: Vec<&str> = components[2].iter().map(GeneId::as_str).collect();

        assert!(module_one.contains(&"g1"));
        assert!(module_one.contains(&"g2"));
        assert!(module_one.contains(&"g3"));
        assert!(module_two.contains(&"g4"));
        assert!(module_two.contains(&"g5"));
        assert_eq!(isolated, vec!["g6"]);
    }

    #[test]
    fn neighbors_return_partner_genes_with_metrics() {
        let index = modular_index();
        let graph =
            CoexpressionGraph::from_index(&index, EdgeFilter::HighestReciprocalRankAtMost(2));
        let mut neighbor_ids: Vec<String> = graph
            .neighbors(&gene("g1"))
            .into_iter()
            .map(|(partner, _)| partner.into_string())
            .collect();
        neighbor_ids.sort();
        assert_eq!(neighbor_ids, vec!["g2", "g3"]);
    }

    #[test]
    fn subgraph_around_collects_within_requested_hops() {
        let index = modular_index();
        let graph =
            CoexpressionGraph::from_index(&index, EdgeFilter::HighestReciprocalRankAtMost(2));

        let one_hop = graph.subgraph_around(&[gene("g1")], 1);
        assert!(one_hop.contains_gene(&gene("g1")));
        assert!(one_hop.contains_gene(&gene("g2")));
        assert!(one_hop.contains_gene(&gene("g3")));
        assert!(!one_hop.contains_gene(&gene("g4")));

        let zero_hop = graph.subgraph_around(&[gene("g1")], 0);
        assert_eq!(zero_hop.node_count(), 1);
        assert_eq!(zero_hop.edge_count(), 0);
    }

    #[test]
    fn mutual_rank_filter_is_symmetric_in_edge_direction() {
        let index = modular_index();
        let graph = CoexpressionGraph::from_index(&index, EdgeFilter::MutualRankAtMost(1.0));
        // g1-g2-g3 all mutually rank 1 with each other => triangle (3 edges).
        let mut neighbors: Vec<String> = graph
            .neighbors(&gene("g2"))
            .into_iter()
            .map(|(partner, _)| partner.into_string())
            .collect();
        neighbors.sort();
        assert_eq!(neighbors, vec!["g1", "g3"]);
    }

    #[test]
    fn missing_gene_neighbors_returns_empty() {
        let index = modular_index();
        let graph =
            CoexpressionGraph::from_index(&index, EdgeFilter::HighestReciprocalRankAtMost(2));
        assert!(graph.neighbors(&gene("ghost")).is_empty());
        assert!(!graph.contains_gene(&gene("ghost")));
    }
}
