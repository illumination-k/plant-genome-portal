/// Benjamini-Hochberg FDR adjustment.
///
/// Returns `q`-values in the same order as the input `p`-values. The
/// adjusted value at position `i` is the smallest BH-step value from
/// rank `i` upward (so monotonicity is preserved). All outputs are
/// clamped to `[0, 1]`.
///
/// Non-finite or out-of-range inputs are passed through to the output —
/// the caller is responsible for filtering those before applying FDR
/// if that is undesired.
pub fn benjamini_hochberg(p_values: &[f64]) -> Vec<f64> {
    let m = p_values.len();
    if m == 0 {
        return Vec::new();
    }

    // Rank ascending by p-value, carrying the original index.
    let mut indexed: Vec<(usize, f64)> = p_values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let m_f = m as f64;
    let mut scaled: Vec<f64> = indexed
        .iter()
        .enumerate()
        .map(|(rank, &(_, p))| {
            let r = (rank + 1) as f64;
            (p * m_f / r).clamp(0.0, 1.0)
        })
        .collect();

    // Enforce monotonic non-decreasing q across ranks by taking the
    // running minimum from the largest p downward.
    for i in (0..scaled.len().saturating_sub(1)).rev() {
        if scaled[i + 1] < scaled[i] {
            scaled[i] = scaled[i + 1];
        }
    }

    let mut q_values = vec![0.0; m];
    for (rank, &(orig_idx, _)) in indexed.iter().enumerate() {
        q_values[orig_idx] = scaled[rank];
    }
    q_values
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn approx_slice(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() <= tol)
    }

    #[test]
    fn empty_input_yields_empty_output() {
        assert!(benjamini_hochberg(&[]).is_empty());
    }

    #[test]
    fn single_value_is_passed_through() {
        let q = benjamini_hochberg(&[0.03]);
        assert_eq!(q, vec![0.03]);
    }

    #[test]
    fn matches_r_p_adjust_bh() {
        // R: p.adjust(c(0.01, 0.04, 0.03, 0.005), method="BH")
        //   -> 0.02, 0.04, 0.04, 0.02
        let q = benjamini_hochberg(&[0.01, 0.04, 0.03, 0.005]);
        assert!(
            approx_slice(&q, &[0.02, 0.04, 0.04, 0.02], 1e-12),
            "got {q:?}"
        );
    }

    #[test]
    fn monotonicity_step_down_correction_applies() {
        // Without the running-min step the raw scaled values for
        // p=[0.1, 0.1, 0.1] would be [0.3, 0.15, 0.1]; after enforcing
        // monotonicity each should collapse to 0.1.
        let q = benjamini_hochberg(&[0.1, 0.1, 0.1]);
        assert!(approx_slice(&q, &[0.1, 0.1, 0.1], 1e-12), "got {q:?}");
    }

    #[test]
    fn q_is_clamped_to_one() {
        // Large p-values with small m can push p * m / r above 1 before
        // clamping; verify nothing escapes [0, 1].
        let q = benjamini_hochberg(&[0.9, 0.95, 1.0]);
        for v in &q {
            assert!((0.0..=1.0).contains(v), "got {v}");
        }
    }

    #[test]
    fn ordering_is_preserved_for_caller() {
        // Input order is shuffled relative to sorted order; output must
        // map back to the original positions, not the sorted ones.
        let p = vec![0.5, 0.01, 0.2];
        let q = benjamini_hochberg(&p);
        // Smallest p is at index 1, so it should receive the smallest q.
        assert!(q[1] < q[2]);
        assert!(q[2] < q[0] || (q[2] - q[0]).abs() < 1e-12);
    }
}
