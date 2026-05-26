//! Co-expression analysis primitives.
//!
//! The crate keeps analysis math out of the HTTP layer. It provides
//! deterministic average-linkage ordering and row-wise z-scores for expression
//! clustergrams, plus ranked gene-pair metrics such as MR, HRR, and LS.

mod error;
mod graph;
mod metrics;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use error::CoexpressionError;
pub use graph::{CoexpressionGraph, EdgeFilter};
pub use metrics::{
    CoexpressionEdge, CoexpressionIndex, CoexpressionMatrices, CoexpressionOptions,
    CorrelationMethod, RankMode, build_coexpression_index, build_coexpression_matrices,
    highest_reciprocal_rank, logit_score, mutual_rank, pearson_correlation, spearman_correlation,
};

/// A flat dendrogram representation suitable for JSON APIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClusterDendrogram {
    pub root: Option<usize>,
    pub nodes: Vec<ClusterDendrogramNode>,
}

impl ClusterDendrogram {
    pub fn leaf_order(&self) -> Vec<usize> {
        let Some(root) = self.root else {
            return Vec::new();
        };
        let mut order = Vec::new();
        self.push_leaf_order(root, &mut order);
        order
    }

    fn push_leaf_order(&self, node_id: usize, order: &mut Vec<usize>) {
        let Some(node) = self.nodes.get(node_id) else {
            return;
        };
        if let Some(leaf_index) = node.leaf_index {
            order.push(leaf_index);
            return;
        }
        if let Some(left) = node.left {
            self.push_leaf_order(left, order);
        }
        if let Some(right) = node.right {
            self.push_leaf_order(right, order);
        }
    }
}

/// One node in a flat dendrogram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClusterDendrogramNode {
    pub id: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub leaf_index: Option<usize>,
    pub distance: f64,
    pub size: usize,
}

/// Computes average-linkage hierarchical ordering for matrix rows.
///
/// `values` is row-major with `row_count * column_count` cells. Missing or
/// malformed cells are treated as zero so API callers can sanitize once at the
/// boundary and still get a stable order.
pub fn cluster_order_rows(values: &[f64], row_count: usize, column_count: usize) -> Vec<usize> {
    cluster_dendrogram_rows(values, row_count, column_count).leaf_order()
}

/// Computes average-linkage hierarchical dendrogram for matrix rows.
pub fn cluster_dendrogram_rows(
    values: &[f64],
    row_count: usize,
    column_count: usize,
) -> ClusterDendrogram {
    let vectors = (0..row_count)
        .map(|row| {
            let start = row.saturating_mul(column_count);
            let end = start.saturating_add(column_count);
            values.get(start..end).unwrap_or_default().to_vec()
        })
        .collect::<Vec<_>>();
    hierarchical_dendrogram(&vectors)
}

/// Computes average-linkage hierarchical ordering for matrix columns.
pub fn cluster_order_columns(values: &[f64], row_count: usize, column_count: usize) -> Vec<usize> {
    cluster_dendrogram_columns(values, row_count, column_count).leaf_order()
}

/// Computes average-linkage hierarchical dendrogram for matrix columns.
pub fn cluster_dendrogram_columns(
    values: &[f64],
    row_count: usize,
    column_count: usize,
) -> ClusterDendrogram {
    let vectors = (0..column_count)
        .map(|column| {
            (0..row_count)
                .map(|row| {
                    values
                        .get(row.saturating_mul(column_count).saturating_add(column))
                        .copied()
                        .unwrap_or(0.0)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    hierarchical_dendrogram(&vectors)
}

/// Computes row-wise z-scores for a row-major matrix.
pub fn row_z_scores(values: &[f64], row_count: usize, column_count: usize) -> Vec<f64> {
    let mut z_scores = Vec::with_capacity(values.len());
    for row in 0..row_count {
        let start = row.saturating_mul(column_count);
        let end = start.saturating_add(column_count);
        let row_values = values.get(start..end).unwrap_or_default();
        let finite = row_values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .collect::<Vec<_>>();
        let mean = finite.iter().sum::<f64>() / finite.len().max(1) as f64;
        let variance = finite
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / finite.len().max(1) as f64;
        let sd = variance.sqrt();
        z_scores.extend(row_values.iter().map(|&value| {
            if sd == 0.0 {
                0.0
            } else {
                (finite_or_zero(value) - mean) / sd
            }
        }));
    }
    z_scores
}

#[derive(Debug, Clone)]
struct Cluster {
    members: Vec<usize>,
    node_id: usize,
}

fn hierarchical_dendrogram(vectors: &[Vec<f64>]) -> ClusterDendrogram {
    let mut nodes = (0..vectors.len())
        .map(|index| ClusterDendrogramNode {
            id: index,
            left: None,
            right: None,
            leaf_index: Some(index),
            distance: 0.0,
            size: 1,
        })
        .collect::<Vec<_>>();
    let mut clusters = (0..vectors.len())
        .map(|index| Cluster {
            members: vec![index],
            node_id: index,
        })
        .collect::<Vec<_>>();

    while clusters.len() > 1 {
        let (left_idx, right_idx, distance) = closest_clusters(vectors, &clusters);
        let right = clusters.remove(right_idx);
        let left = clusters.remove(left_idx);
        clusters.push(merge_clusters(left, right, distance, &mut nodes));
    }

    ClusterDendrogram {
        root: clusters.pop().map(|cluster| cluster.node_id),
        nodes,
    }
}

fn closest_clusters(vectors: &[Vec<f64>], clusters: &[Cluster]) -> (usize, usize, f64) {
    let mut best = (0, 1);
    let mut best_distance = f64::INFINITY;

    for left in 0..clusters.len() {
        for right in (left + 1)..clusters.len() {
            let distance = average_linkage_distance(vectors, &clusters[left], &clusters[right]);
            if distance < best_distance {
                best = (left, right);
                best_distance = distance;
            }
        }
    }

    (best.0, best.1, best_distance)
}

fn average_linkage_distance(vectors: &[Vec<f64>], left: &Cluster, right: &Cluster) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for &left_member in &left.members {
        for &right_member in &right.members {
            total += euclidean_distance(&vectors[left_member], &vectors[right_member]);
            count += 1;
        }
    }
    total / count.max(1) as f64
}

fn merge_clusters(
    left: Cluster,
    right: Cluster,
    distance: f64,
    nodes: &mut Vec<ClusterDendrogramNode>,
) -> Cluster {
    let (mut first, second, first_node_id, second_node_id) =
        match (left.members.first(), right.members.first()) {
            (Some(left_first), Some(right_first)) if left_first <= right_first => {
                (left.members, right.members, left.node_id, right.node_id)
            }
            _ => (right.members, left.members, right.node_id, left.node_id),
        };
    let size = first.len() + second.len();
    let node_id = nodes.len();
    nodes.push(ClusterDendrogramNode {
        id: node_id,
        left: Some(first_node_id),
        right: Some(second_node_id),
        leaf_index: None,
        distance,
        size,
    });
    first.extend(second);
    Cluster {
        members: first,
        node_id,
    }
}

fn euclidean_distance(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(&left_value, &right_value)| {
            let delta = finite_or_zero(left_value) - finite_or_zero(right_value);
            delta * delta
        })
        .sum::<f64>()
        .sqrt()
}

fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn row_z_scores_normalize_each_gene() {
        let values = vec![1.0, 2.0, 3.0, 10.0, 10.0, 10.0];

        let z_scores = row_z_scores(&values, 2, 3);

        assert!((z_scores[0] + 1.224_744_871).abs() < 0.000_001);
        assert_eq!(z_scores[1], 0.0);
        assert!((z_scores[2] - 1.224_744_871).abs() < 0.000_001);
        assert_eq!(&z_scores[3..], &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn row_order_keeps_similar_profiles_adjacent() {
        let values = vec![
            1.0, 1.0, 1.0, //
            10.0, 10.0, 10.0, //
            1.2, 1.1, 1.0,
        ];

        let order = cluster_order_rows(&values, 3, 3);
        let pos0 = order.iter().position(|index| *index == 0).unwrap();
        let pos2 = order.iter().position(|index| *index == 2).unwrap();

        assert_eq!(pos0.abs_diff(pos2), 1);
    }

    #[test]
    fn column_order_keeps_similar_samples_adjacent() {
        let values = vec![
            1.0, 1.1, 10.0, //
            2.0, 2.1, 20.0,
        ];

        let order = cluster_order_columns(&values, 2, 3);
        let pos0 = order.iter().position(|index| *index == 0).unwrap();
        let pos1 = order.iter().position(|index| *index == 1).unwrap();

        assert_eq!(pos0.abs_diff(pos1), 1);
    }

    #[test]
    fn dendrogram_records_internal_merge_distances() {
        let values = vec![
            1.0, 1.0, //
            1.2, 1.1, //
            10.0, 10.0,
        ];

        let dendrogram = cluster_dendrogram_rows(&values, 3, 2);

        assert_eq!(dendrogram.nodes.len(), 5);
        assert_eq!(dendrogram.root, Some(4));
        assert_eq!(dendrogram.leaf_order().len(), 3);
        assert!(dendrogram.nodes[3].distance > 0.0);
        assert!(dendrogram.nodes[4].distance > dendrogram.nodes[3].distance);
    }
}
