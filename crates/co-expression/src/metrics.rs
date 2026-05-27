use std::cmp::Ordering;

use blas_src as _;
use expression_core::ExpressionMatrix;
use genome_core::{AssemblyAccession, GeneId};
use ndarray::Array2;
use ndarray_stats::CorrelationExt;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::CoexpressionError;

/// Correlation method used before rank transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CorrelationMethod {
    Pearson,
    Spearman,
}

/// How correlations are ordered when assigning per-gene ranks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RankMode {
    /// Larger positive correlations rank first.
    Positive,
    /// Larger absolute correlations rank first, keeping anti-correlated pairs.
    Absolute,
}

impl RankMode {
    fn score(self, correlation: f64) -> f64 {
        match self {
            Self::Positive => correlation,
            Self::Absolute => correlation.abs(),
        }
    }
}

/// Options for building a co-expression network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CoexpressionOptions {
    pub correlation_method: CorrelationMethod,
    pub rank_mode: RankMode,
}

impl Default for CoexpressionOptions {
    fn default() -> Self {
        Self {
            correlation_method: CorrelationMethod::Pearson,
            rank_mode: RankMode::Positive,
        }
    }
}

/// A ranked, undirected gene-pair edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CoexpressionEdge {
    pub gene_a: GeneId,
    pub gene_b: GeneId,
    pub correlation: f64,
    pub rank_a_to_b: usize,
    pub rank_b_to_a: usize,
    pub mutual_rank: f64,
    pub highest_reciprocal_rank: usize,
    pub logit_score: f64,
}

/// Complete dense co-expression matrices.
///
/// This stores row-major `gene_count * gene_count` matrices for correlation,
/// directional rank, MR, HRR, and LS. For 20k genes this is intentionally a
/// multi-GB structure; it avoids per-edge allocation and repeated `GeneId`
/// cloning while still computing every ordered pair.
#[derive(Debug, Clone, PartialEq)]
pub struct CoexpressionIndex {
    pub assembly_accession: AssemblyAccession,
    pub gene_ids: Vec<GeneId>,
    pub correlation_method: CorrelationMethod,
    pub rank_mode: RankMode,
    correlations: Vec<f32>,
    ranks: Vec<u32>,
    mutual_ranks: Vec<f32>,
    highest_reciprocal_ranks: Vec<u32>,
    logit_scores: Vec<f32>,
}

pub type CoexpressionMatrices = CoexpressionIndex;

impl CoexpressionIndex {
    pub fn gene_count(&self) -> usize {
        self.gene_ids.len()
    }

    pub fn correlations(&self) -> &[f32] {
        &self.correlations
    }

    pub fn directional_ranks(&self) -> &[u32] {
        &self.ranks
    }

    pub fn mutual_ranks(&self) -> &[f32] {
        &self.mutual_ranks
    }

    pub fn highest_reciprocal_ranks(&self) -> &[u32] {
        &self.highest_reciprocal_ranks
    }

    pub fn logit_scores(&self) -> &[f32] {
        &self.logit_scores
    }

    fn pair_offset(&self, source: usize, target: usize) -> Option<usize> {
        let gene_count = self.gene_count();
        (source != target && source < gene_count && target < gene_count)
            .then_some(source * gene_count + target)
    }

    pub fn correlation_by_index(&self, source: usize, target: usize) -> Option<f64> {
        let correlation = f64::from(self.correlations[self.pair_offset(source, target)?]);
        correlation.is_finite().then_some(correlation)
    }

    pub fn rank_by_index(&self, source: usize, target: usize) -> Option<usize> {
        let rank = self.ranks[self.pair_offset(source, target)?];
        (rank > 0).then_some(rank as usize)
    }

    pub fn mutual_rank_by_index(&self, source: usize, target: usize) -> Option<f64> {
        let value = f64::from(self.mutual_ranks[self.pair_offset(source, target)?]);
        value.is_finite().then_some(value)
    }

    pub fn highest_reciprocal_rank_by_index(&self, source: usize, target: usize) -> Option<usize> {
        let value = self.highest_reciprocal_ranks[self.pair_offset(source, target)?];
        (value > 0).then_some(value as usize)
    }

    pub fn logit_score_by_index(&self, source: usize, target: usize) -> Option<f64> {
        let value = f64::from(self.logit_scores[self.pair_offset(source, target)?]);
        value.is_finite().then_some(value)
    }

    pub fn edge_by_index(&self, gene_a: usize, gene_b: usize) -> Option<CoexpressionEdge> {
        let correlation = self.correlation_by_index(gene_a, gene_b)?;
        let rank_a_to_b = self.rank_by_index(gene_a, gene_b)?;
        let rank_b_to_a = self.rank_by_index(gene_b, gene_a)?;
        Some(CoexpressionEdge {
            gene_a: self.gene_ids[gene_a].clone(),
            gene_b: self.gene_ids[gene_b].clone(),
            correlation,
            rank_a_to_b,
            rank_b_to_a,
            mutual_rank: self.mutual_rank_by_index(gene_a, gene_b)?,
            highest_reciprocal_rank: self.highest_reciprocal_rank_by_index(gene_a, gene_b)?,
            logit_score: self.logit_score_by_index(gene_a, gene_b)?,
        })
    }

    pub fn edge_by_gene_id(&self, gene_a: &GeneId, gene_b: &GeneId) -> Option<CoexpressionEdge> {
        let gene_a = self.gene_ids.iter().position(|id| id == gene_a)?;
        let gene_b = self.gene_ids.iter().position(|id| id == gene_b)?;
        self.edge_by_index(gene_a, gene_b)
    }
}

/// Computes Pearson's correlation coefficient for two vectors.
///
/// Non-finite pairs are skipped so `NaN` can represent missing expression
/// measurements. Returns `Ok(None)` when fewer than two finite pairs remain or
/// either vector has zero variance.
pub fn pearson_correlation(left: &[f64], right: &[f64]) -> Result<Option<f64>, CoexpressionError> {
    if left.len() != right.len() {
        return Err(CoexpressionError::VectorLengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }

    let mut count = 0.0;
    let mut mean_left = 0.0;
    let mut mean_right = 0.0;
    let mut sum_left_sq = 0.0;
    let mut sum_right_sq = 0.0;
    let mut sum_cross = 0.0;

    for (&left_value, &right_value) in left.iter().zip(right) {
        if !left_value.is_finite() || !right_value.is_finite() {
            continue;
        }

        count += 1.0;
        let delta_left = left_value - mean_left;
        mean_left += delta_left / count;
        let delta_right = right_value - mean_right;
        mean_right += delta_right / count;
        sum_left_sq += delta_left * (left_value - mean_left);
        sum_right_sq += delta_right * (right_value - mean_right);
        sum_cross += delta_left * (right_value - mean_right);
    }

    if count < 2.0 || sum_left_sq <= 0.0 || sum_right_sq <= 0.0 {
        return Ok(None);
    }

    Ok(Some(
        (sum_cross / (sum_left_sq.sqrt() * sum_right_sq.sqrt())).clamp(-1.0, 1.0),
    ))
}

/// Computes Spearman's rank correlation coefficient for two vectors.
///
/// Non-finite pairs are skipped before ranking. Tied values receive average
/// one-based ranks.
pub fn spearman_correlation(left: &[f64], right: &[f64]) -> Result<Option<f64>, CoexpressionError> {
    if left.len() != right.len() {
        return Err(CoexpressionError::VectorLengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }

    let mut paired_left = Vec::new();
    let mut paired_right = Vec::new();
    for (&left_value, &right_value) in left.iter().zip(right) {
        if left_value.is_finite() && right_value.is_finite() {
            paired_left.push(left_value);
            paired_right.push(right_value);
        }
    }

    if paired_left.len() < 2 {
        return Ok(None);
    }

    let left_ranks = average_ranks(&paired_left);
    let right_ranks = average_ranks(&paired_right);
    pearson_correlation(&left_ranks, &right_ranks)
}

/// Mutual Rank (MR): geometric mean of the two directional ranks.
pub fn mutual_rank(rank_a_to_b: usize, rank_b_to_a: usize) -> f64 {
    ((rank_a_to_b as f64) * (rank_b_to_a as f64)).sqrt()
}

/// Highest Reciprocal Rank (HRR): the larger of two directional ranks.
pub fn highest_reciprocal_rank(rank_a_to_b: usize, rank_b_to_a: usize) -> usize {
    rank_a_to_b.max(rank_b_to_a)
}

/// Logit Score (LS): `-log2(p / (1 - p))`, where `p = MR / gene_count`.
pub fn logit_score(mutual_rank: f64, gene_count: usize) -> f64 {
    let p = mutual_rank / (gene_count as f64);
    -(p / (1.0 - p)).log2()
}

/// Builds complete dense co-expression matrices using BLAS GEMM and rayon.
///
/// Pearson uses `ndarray-stats`; Spearman ranks rows first, then uses the same
/// `ndarray-stats` Pearson path. With ndarray's BLAS feature enabled, the
/// matrix-heavy part is delegated through ndarray's BLAS backend. Pairs
/// involving non-finite values fall back to the exact pairwise functions,
/// preserving missing-value semantics.
pub fn build_coexpression_index(
    matrix: &ExpressionMatrix,
    options: CoexpressionOptions,
) -> Result<CoexpressionIndex, CoexpressionError> {
    build_coexpression_matrices(matrix, options)
}

pub fn build_coexpression_matrices(
    matrix: &ExpressionMatrix,
    options: CoexpressionOptions,
) -> Result<CoexpressionMatrices, CoexpressionError> {
    validate_matrix(matrix)?;

    let gene_count = matrix.gene_count();
    if gene_count > u32::MAX as usize {
        return Err(CoexpressionError::TooManyGenesForRank { gene_count });
    }
    let total_cells = gene_count
        .checked_mul(gene_count)
        .ok_or(CoexpressionError::MatrixTooLarge)?;
    let prepared = prepare_correlation_input(matrix, options.correlation_method)?;
    let mut correlations = correlation_matrix(&prepared)?;
    let mut ranks = vec![0_u32; total_cells];

    fill_exact_fallback_correlations(&mut correlations, matrix, &prepared, options);
    rank_correlation_targets(&correlations, &mut ranks, gene_count, options.rank_mode);

    let mut mutual_ranks = vec![f32::NAN; total_cells];
    let mut highest_reciprocal_ranks = vec![0_u32; total_cells];
    let mut logit_scores = vec![f32::NAN; total_cells];
    fill_rank_scores(
        &ranks,
        gene_count,
        &mut mutual_ranks,
        &mut highest_reciprocal_ranks,
        &mut logit_scores,
    );

    Ok(CoexpressionIndex {
        assembly_accession: matrix.assembly_accession.clone(),
        gene_ids: matrix.gene_ids.clone(),
        correlation_method: options.correlation_method,
        rank_mode: options.rank_mode,
        correlations,
        ranks,
        mutual_ranks,
        highest_reciprocal_ranks,
        logit_scores,
    })
}

fn rank_correlation_targets(
    correlations: &[f32],
    ranks: &mut [u32],
    gene_count: usize,
    rank_mode: RankMode,
) {
    ranks
        .par_chunks_mut(gene_count.max(1))
        .enumerate()
        .for_each(|(source, rank_row)| {
            if gene_count == 0 {
                return;
            }

            let correlation_row = raw_matrix_row(correlations, source, gene_count);
            let mut scored_targets = Vec::with_capacity(gene_count.saturating_sub(1));
            for (target, &correlation) in correlation_row.iter().enumerate() {
                if source == target {
                    continue;
                }
                if correlation.is_finite() {
                    scored_targets.push((target, rank_mode.score(f64::from(correlation))));
                }
            }

            assign_ranks(&mut scored_targets, rank_row);
        });
}

fn fill_rank_scores(
    ranks: &[u32],
    gene_count: usize,
    mutual_ranks: &mut [f32],
    highest_reciprocal_ranks: &mut [u32],
    logit_scores: &mut [f32],
) {
    mutual_ranks
        .par_chunks_mut(gene_count.max(1))
        .zip(highest_reciprocal_ranks.par_chunks_mut(gene_count.max(1)))
        .zip(logit_scores.par_chunks_mut(gene_count.max(1)))
        .enumerate()
        .for_each(|(source, ((mutual_rank_row, hrr_row), logit_score_row))| {
            for target in 0..gene_count {
                fill_rank_score_cell(
                    ranks,
                    gene_count,
                    source,
                    target,
                    mutual_rank_row,
                    hrr_row,
                    logit_score_row,
                );
            }
        });
}

fn fill_rank_score_cell(
    ranks: &[u32],
    gene_count: usize,
    source: usize,
    target: usize,
    mutual_rank_row: &mut [f32],
    hrr_row: &mut [u32],
    logit_score_row: &mut [f32],
) {
    if source == target {
        return;
    }
    let rank_source_to_target = ranks[source * gene_count + target];
    let rank_target_to_source = ranks[target * gene_count + source];
    if rank_source_to_target == 0 || rank_target_to_source == 0 {
        return;
    }

    let mr = mutual_rank(
        rank_source_to_target as usize,
        rank_target_to_source as usize,
    );
    mutual_rank_row[target] = mr as f32;
    hrr_row[target] = highest_reciprocal_rank(
        rank_source_to_target as usize,
        rank_target_to_source as usize,
    ) as u32;
    logit_score_row[target] = logit_score(mr, gene_count) as f32;
}

fn assign_ranks(scored_targets: &mut [(usize, f64)], rank_row: &mut [u32]) {
    scored_targets.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut current_rank = 0_u32;
    let mut previous_score = None;
    for (position, &(target, score)) in scored_targets.iter().enumerate() {
        if previous_score.is_none_or(|previous| score.total_cmp(&previous) != Ordering::Equal) {
            current_rank = (position + 1) as u32;
            previous_score = Some(score);
        }
        rank_row[target] = current_rank;
    }
}

struct PreparedRows {
    values: Array2<f32>,
    valid: Vec<bool>,
}

fn prepare_correlation_input(
    matrix: &ExpressionMatrix,
    method: CorrelationMethod,
) -> Result<PreparedRows, CoexpressionError> {
    let column_count = matrix.run_count();
    if column_count == 0 {
        return Ok(PreparedRows {
            values: Array2::zeros((matrix.gene_count(), 0)),
            valid: vec![false; matrix.gene_count()],
        });
    }

    let prepared = matrix
        .values
        .par_chunks(column_count)
        .map(|row| match method {
            CorrelationMethod::Pearson => finite_row_as_f32(row),
            CorrelationMethod::Spearman => {
                if row.iter().all(|value| value.is_finite()) {
                    (
                        average_ranks(row)
                            .into_iter()
                            .map(|value| value as f32)
                            .collect::<Vec<_>>(),
                        true,
                    )
                } else {
                    (vec![0.0; row.len()], false)
                }
            }
        })
        .collect::<Vec<_>>();

    let mut values = Vec::with_capacity(matrix.values.len());
    let mut valid = Vec::with_capacity(matrix.gene_count());
    for (row, row_is_valid) in prepared {
        values.extend(row);
        valid.push(row_is_valid);
    }

    let values = Array2::from_shape_vec((matrix.gene_count(), column_count), values)
        .map_err(|_| CoexpressionError::MatrixLayout)?;
    Ok(PreparedRows { values, valid })
}

fn finite_row_as_f32(row: &[f64]) -> (Vec<f32>, bool) {
    if row.iter().any(|value| !value.is_finite()) {
        return (vec![0.0; row.len()], false);
    }
    (
        row.iter().map(|&value| value as f32).collect::<Vec<_>>(),
        true,
    )
}

fn correlation_matrix(prepared: &PreparedRows) -> Result<Vec<f32>, CoexpressionError> {
    let product = prepared
        .values
        .pearson_correlation()
        .map_err(|_| CoexpressionError::EmptyCorrelationInput)?;
    let (mut values, offset) = product.into_raw_vec_and_offset();
    if offset.is_some_and(|offset| offset != 0) {
        return Err(CoexpressionError::MatrixLayout);
    }
    let gene_count = prepared.valid.len();
    for gene in 0..gene_count {
        values[gene * gene_count + gene] = f32::NAN;
    }
    Ok(values)
}

fn fill_exact_fallback_correlations(
    correlations: &mut [f32],
    matrix: &ExpressionMatrix,
    prepared: &PreparedRows,
    options: CoexpressionOptions,
) {
    if prepared.valid.iter().all(|&valid| valid) {
        return;
    }

    let gene_count = matrix.gene_count();
    correlations
        .par_chunks_mut(gene_count.max(1))
        .enumerate()
        .for_each(|(source, correlation_row)| {
            let source_raw_row = raw_matrix_row(&matrix.values, source, matrix.run_count());
            for (target, correlation_cell) in correlation_row.iter_mut().enumerate() {
                if source == target || (prepared.valid[source] && prepared.valid[target]) {
                    continue;
                }

                *correlation_cell = exact_correlation(
                    source_raw_row,
                    raw_matrix_row(&matrix.values, target, matrix.run_count()),
                    options.correlation_method,
                )
                .map(|correlation| correlation.clamp(-1.0, 1.0) as f32)
                .unwrap_or(f32::NAN);
            }
        });
}

fn raw_matrix_row<T>(values: &[T], row: usize, column_count: usize) -> &[T] {
    let start = row * column_count;
    let end = start + column_count;
    &values[start..end]
}

fn exact_correlation(left: &[f64], right: &[f64], method: CorrelationMethod) -> Option<f64> {
    match method {
        CorrelationMethod::Pearson => pearson_correlation(left, right),
        CorrelationMethod::Spearman => spearman_correlation(left, right),
    }
    .ok()
    .flatten()
}

fn validate_matrix(matrix: &ExpressionMatrix) -> Result<(), CoexpressionError> {
    let expected = matrix
        .gene_count()
        .checked_mul(matrix.run_count())
        .ok_or(CoexpressionError::MatrixTooLarge)?;
    if matrix.values.len() != expected {
        return Err(CoexpressionError::ExpressionMatrixDimensionMismatch {
            expected,
            actual: matrix.values.len(),
        });
    }
    Ok(())
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut indexed_values: Vec<_> = values.iter().copied().enumerate().collect();
    indexed_values.sort_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
    });

    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < indexed_values.len() {
        let mut end = start + 1;
        while end < indexed_values.len()
            && indexed_values[end].1.total_cmp(&indexed_values[start].1) == Ordering::Equal
        {
            end += 1;
        }

        let average_rank = ((start + 1 + end) as f64) / 2.0;
        for &(original_index, _) in &indexed_values[start..end] {
            ranks[original_index] = average_rank;
        }
        start = end;
    }

    ranks
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use expression_core::{ExpressionMatrix, ExpressionUnit, SraRunAccession};

    fn assembly() -> AssemblyAccession {
        AssemblyAccession::new("GCA_037833805.1").unwrap()
    }

    fn gene(id: &str) -> GeneId {
        GeneId::new(id).unwrap()
    }

    fn run(id: &str) -> SraRunAccession {
        SraRunAccession::new(id).unwrap()
    }

    fn runs(count: usize) -> Vec<SraRunAccession> {
        (1..=count)
            .map(|index| run(&format!("SRR{index:06}")))
            .collect()
    }

    fn matrix(values: Vec<f64>) -> ExpressionMatrix {
        ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![gene("g1"), gene("g2"), gene("g3")],
            runs(3),
            values,
        )
        .unwrap()
    }

    #[test]
    fn pearson_skips_missing_pairs() {
        let correlation = pearson_correlation(&[1.0, f64::NAN, 3.0, 4.0], &[2.0, 9.0, 6.0, 8.0])
            .unwrap()
            .unwrap();
        assert!((correlation - 1.0).abs() < 1e-12);
    }

    #[test]
    fn pearson_returns_none_for_zero_variance() {
        assert_eq!(
            pearson_correlation(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]).unwrap(),
            None
        );
    }

    #[test]
    fn spearman_uses_average_ranks_for_ties() {
        let ranks = average_ranks(&[10.0, 20.0, 20.0, 40.0]);
        assert_eq!(ranks, vec![1.0, 2.5, 2.5, 4.0]);

        let correlation = spearman_correlation(&[10.0, 20.0, 20.0, 40.0], &[1.0, 2.0, 2.0, 3.0])
            .unwrap()
            .unwrap();
        assert!((correlation - 1.0).abs() < 1e-12);
    }

    #[test]
    fn metric_helpers_match_rank_definitions() {
        assert!((mutual_rank(4, 9) - 6.0).abs() < 1e-12);
        assert_eq!(highest_reciprocal_rank(4, 9), 9);
        assert!((logit_score(25.0, 100) - 1.584_962_500_721_156_3).abs() < 1e-12);
    }

    #[test]
    fn can_read_ranked_edge_from_dense_matrices() {
        let matrix = matrix(vec![
            1.0, 2.0, 3.0, // g1
            2.0, 4.0, 6.0, // g2
            3.0, 2.0, 1.0, // g3
        ]);

        let matrices =
            build_coexpression_matrices(&matrix, CoexpressionOptions::default()).unwrap();

        let edge = matrices.edge_by_index(0, 1).unwrap();
        assert_eq!(edge.gene_a.as_str(), "g1");
        assert_eq!(edge.gene_b.as_str(), "g2");
        assert!((edge.correlation - 1.0).abs() < 1e-6);
        assert_eq!(edge.rank_a_to_b, 1);
        assert_eq!(edge.rank_b_to_a, 1);
        assert!((edge.mutual_rank - 1.0).abs() < 1e-12);
        assert_eq!(edge.highest_reciprocal_rank, 1);
    }

    #[test]
    fn builds_complete_dense_metric_matrices() {
        let matrix = matrix(vec![
            1.0, 2.0, 3.0, // g1
            2.0, 4.0, 6.0, // g2
            3.0, 2.0, 1.0, // g3
        ]);

        let matrices =
            build_coexpression_matrices(&matrix, CoexpressionOptions::default()).unwrap();

        assert_eq!(matrices.correlations().len(), 9);
        assert_eq!(matrices.directional_ranks().len(), 9);
        assert_eq!(matrices.mutual_ranks().len(), 9);
        assert_eq!(matrices.highest_reciprocal_ranks().len(), 9);
        assert_eq!(matrices.logit_scores().len(), 9);
        assert!((matrices.correlation_by_index(0, 1).unwrap() - 1.0).abs() < 1e-6);
        assert_eq!(matrices.rank_by_index(0, 1), Some(1));
        assert!((matrices.mutual_rank_by_index(0, 1).unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(matrices.highest_reciprocal_rank_by_index(0, 1), Some(1));
        assert!(matrices.logit_score_by_index(0, 1).unwrap().is_finite());
    }

    #[test]
    fn dense_matrices_fall_back_to_exact_missing_value_correlations() {
        let matrix = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![gene("g1"), gene("g2"), gene("g3")],
            runs(4),
            vec![
                1.0,
                f64::NAN,
                3.0,
                4.0, // g1
                2.0,
                9.0,
                6.0,
                8.0, // g2
                1.0,
                1.0,
                1.0,
                1.0, // g3
            ],
        )
        .unwrap();

        let matrices =
            build_coexpression_matrices(&matrix, CoexpressionOptions::default()).unwrap();

        assert!((matrices.correlation_by_index(0, 1).unwrap() - 1.0).abs() < 1e-12);
        assert_eq!(matrices.correlation_by_index(0, 2), None);
    }

    #[test]
    fn absolute_rank_mode_keeps_strong_negative_pairs_near_top() {
        let matrix = matrix(vec![
            1.0, 2.0, 3.0, // g1
            3.0, 2.0, 1.0, // g2
            2.0, 2.0, 3.0, // g3
        ]);

        let matrices = build_coexpression_matrices(
            &matrix,
            CoexpressionOptions {
                correlation_method: CorrelationMethod::Pearson,
                rank_mode: RankMode::Absolute,
            },
        )
        .unwrap();

        let edge = matrices.edge_by_index(0, 1).unwrap();
        assert_eq!(edge.gene_a.as_str(), "g1");
        assert_eq!(edge.gene_b.as_str(), "g2");
        assert!((edge.correlation + 1.0).abs() < 1e-6);
        assert_eq!(edge.rank_a_to_b, 1);
        assert_eq!(edge.rank_b_to_a, 1);
    }

    #[test]
    fn empty_or_single_gene_matrix_has_no_edges() {
        let matrix = ExpressionMatrix::new(
            assembly(),
            ExpressionUnit::Tpm,
            vec![gene("g1")],
            vec![run("SRR000001")],
            vec![1.0],
        )
        .unwrap();

        let matrices =
            build_coexpression_matrices(&matrix, CoexpressionOptions::default()).unwrap();
        assert_eq!(matrices.correlations().len(), 1);
        assert_eq!(matrices.edge_by_index(0, 0), None);
    }

    #[test]
    fn fill_rank_score_cell_uses_symmetric_dense_rank_offsets() {
        let gene_count = 10;
        let mut ranks = vec![0; gene_count * gene_count];
        ranks[1] = 2;
        ranks[10] = 8;
        let mut mutual_rank_row = vec![0.0; gene_count];
        let mut hrr_row = vec![0; gene_count];
        let mut logit_score_row = vec![0.0; gene_count];

        fill_rank_score_cell(
            &ranks,
            gene_count,
            0,
            1,
            &mut mutual_rank_row,
            &mut hrr_row,
            &mut logit_score_row,
        );

        assert!((mutual_rank_row[1] - 4.0).abs() < 1e-6);
        assert_eq!(hrr_row[1], 8);
        assert!((logit_score_row[1] - logit_score(4.0, gene_count) as f32).abs() < 1e-6);
    }

    #[test]
    fn fill_rank_score_cell_leaves_missing_direction_unscored() {
        let gene_count = 10;
        let mut ranks = vec![0; gene_count * gene_count];
        ranks[1] = 2;
        let mut mutual_rank_row = vec![99.0; gene_count];
        let mut hrr_row = vec![99; gene_count];
        let mut logit_score_row = vec![99.0; gene_count];

        fill_rank_score_cell(
            &ranks,
            gene_count,
            0,
            1,
            &mut mutual_rank_row,
            &mut hrr_row,
            &mut logit_score_row,
        );

        assert_eq!(mutual_rank_row[1], 99.0);
        assert_eq!(hrr_row[1], 99);
        assert_eq!(logit_score_row[1], 99.0);
    }
}
