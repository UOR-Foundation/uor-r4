//! e-means: mini-batch k-means with exponential-moving-average centroids.
//!
//! Bessa, Mazaré, Szilvasy, Touvron, Picard, Douze and Jégou, "e-means:
//! outperforming k-means (in one-tenth the epochs)" (2026). Each cluster keeps
//! a count `N_j` and a sum `S_j`, updated as `N_j <- gamma N_j + (1-gamma) n_j`
//! and `S_j <- gamma S_j + (1-gamma) s_j`; the centroid is always read as
//! `S_j / N_j`, never stored separately. The step `alpha = 1 - gamma` is
//! cosine-annealed, a cluster whose count falls below a tenth of the mean is
//! revived from the farthest point, and the last epoch's exact per-cluster
//! means are the returned centroids.
//!
//! This is an offline, host-side codebook fit, on the same `f64` scale as the
//! greedy placement it can replace. It changes the solver, not the table
//! format: the caller still quantizes to the integer boundary itself. The
//! paper's scaling laws are not adopted at this scale; the constants are fixed.
//!
//! Deterministic for a seed: the initialization, the shuffle and the batches
//! all draw from the module's xorshift.

/// Epochs a fit runs: four is the safe end of the paper's 1.4--2.6 range at
/// these sample sizes.
pub const EPOCHS: usize = 4;
/// Mini-batches an epoch. This is a fixed choice, not the paper's batch law.
pub const BATCHES: usize = 32;
/// The cosine schedule's peak step.
pub const ALPHA0: f64 = 0.3;

fn next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn uniform(state: &mut u64) -> f64 {
    (next(state) >> 11) as f64 / (1u64 << 53) as f64
}

fn distance2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum()
}

fn nearest(centroids: &[f64], dim: usize, point: &[f64]) -> usize {
    let mut best = 0usize;
    let mut best_distance = f64::INFINITY;
    for (code, centroid) in centroids.chunks_exact(dim).enumerate() {
        let distance = distance2(centroid, point);
        if distance < best_distance {
            best_distance = distance;
            best = code;
        }
    }
    best
}

/// Seeded k-means++ initialization: the first centroid is a uniform row and
/// each next one is drawn with probability proportional to its squared
/// distance from the nearest chosen centroid. `k` centroids of `dim` values.
pub fn init_centroids(points: &[f64], n: usize, dim: usize, k: usize, seed: u64) -> Vec<f64> {
    let mut state = seed | 1;
    let mut centroids = Vec::with_capacity(k * dim);
    let first = (next(&mut state) as usize) % n;
    centroids.extend_from_slice(&points[first * dim..(first + 1) * dim]);
    let mut closest = vec![f64::INFINITY; n];
    for _ in 1..k {
        let start = centroids.len() - dim;
        let chosen = &centroids[start..];
        let mut total = 0.0f64;
        for index in 0..n {
            let point = &points[index * dim..(index + 1) * dim];
            let distance = distance2(point, chosen).min(closest[index]);
            closest[index] = distance;
            total += distance;
        }
        let target = uniform(&mut state) * total;
        let mut cumulative = 0.0f64;
        let mut index = n - 1;
        for (row, &weight) in closest.iter().enumerate() {
            cumulative += weight;
            if cumulative >= target {
                index = row;
                break;
            }
        }
        centroids.extend_from_slice(&points[index * dim..(index + 1) * dim]);
    }
    centroids
}

/// Fit `k` centroids of `dim` coordinates on `n` flat points, returning
/// `k * dim` coordinates in row-major order.
pub fn fit(
    points: &[f64],
    n: usize,
    dim: usize,
    k: usize,
    epochs: usize,
    seed: u64,
) -> Result<Vec<f64>, String> {
    if n == 0 || dim == 0 || k == 0 {
        return Err("e-means needs points, dimensions and clusters".to_owned());
    }
    if points.len() != n * dim {
        return Err("e-means points are n * dim".to_owned());
    }
    let mut state = seed | 1;
    let mut centroids = init_centroids(points, n, dim, k, seed);
    let mut counts = vec![1.0f64; k];
    let mut sums = centroids.clone();
    let mut order: Vec<usize> = (0..n).collect();
    let batches = BATCHES.min(n).max(1);
    let total = (epochs * batches).max(1);
    let mut step = 0usize;
    for _ in 0..epochs {
        for i in (1..n).rev() {
            let j = (next(&mut state) as usize) % (i + 1);
            order.swap(i, j);
        }
        for batch in 0..batches {
            let start = batch * n / batches;
            let end = (batch + 1) * n / batches;
            let progress = step as f64 / total as f64;
            let alpha =
                ALPHA0 * (0.1 + 0.9 * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos()));
            let gamma = 1.0 - alpha;
            let mut batch_sums = vec![0.0f64; k * dim];
            let mut batch_counts = vec![0usize; k];
            for &index in &order[start..end] {
                let point = &points[index * dim..(index + 1) * dim];
                let code = nearest(&centroids, dim, point);
                batch_counts[code] += 1;
                for j in 0..dim {
                    batch_sums[code * dim + j] += point[j];
                }
            }
            for code in 0..k {
                if batch_counts[code] == 0 {
                    continue;
                }
                counts[code] = gamma * counts[code] + alpha * batch_counts[code] as f64;
                for j in 0..dim {
                    sums[code * dim + j] =
                        gamma * sums[code * dim + j] + alpha * batch_sums[code * dim + j];
                }
                let inv = 1.0 / counts[code];
                for j in 0..dim {
                    centroids[code * dim + j] = sums[code * dim + j] * inv;
                }
            }
            step += 1;
            if step.is_multiple_of(5) {
                revive(&mut centroids, &mut counts, &mut sums, points, n, dim, k);
            }
        }
    }
    // Last-epoch mean: the exact mean of every point's final assignment.
    let mut final_sums = vec![0.0f64; k * dim];
    let mut final_counts = vec![0usize; k];
    for index in 0..n {
        let point = &points[index * dim..(index + 1) * dim];
        let code = nearest(&centroids, dim, point);
        final_counts[code] += 1;
        for j in 0..dim {
            final_sums[code * dim + j] += point[j];
        }
    }
    for code in 0..k {
        if final_counts[code] == 0 {
            continue;
        }
        let inv = 1.0 / final_counts[code] as f64;
        for j in 0..dim {
            centroids[code * dim + j] = final_sums[code * dim + j] * inv;
        }
    }
    Ok(centroids)
}

/// Mean squared distance from every point to its nearest centroid.
pub fn mse(points: &[f64], n: usize, dim: usize, centroids: &[f64]) -> f64 {
    let total: f64 = (0..n)
        .map(|index| {
            let point = &points[index * dim..(index + 1) * dim];
            centroids
                .chunks_exact(dim)
                .map(|centroid| distance2(centroid, point))
                .fold(f64::INFINITY, f64::min)
        })
        .sum();
    total / n as f64
}

/// Rebuild every cluster whose EMA count fell below a tenth of the mean from
/// the point farthest from its nearest centroid.
fn revive(
    centroids: &mut [f64],
    counts: &mut [f64],
    sums: &mut [f64],
    points: &[f64],
    n: usize,
    dim: usize,
    k: usize,
) {
    let mean: f64 = counts.iter().sum::<f64>() / k as f64;
    for code in 0..k {
        if counts[code] >= 0.1 * mean {
            continue;
        }
        let mut farthest = 0usize;
        let mut farthest_distance = -1.0f64;
        for index in 0..n {
            let point = &points[index * dim..(index + 1) * dim];
            let distance = centroids
                .chunks_exact(dim)
                .map(|centroid| distance2(centroid, point))
                .fold(f64::INFINITY, f64::min);
            if distance > farthest_distance {
                farthest_distance = distance;
                farthest = index;
            }
        }
        let source = farthest * dim;
        centroids[code * dim..(code + 1) * dim].copy_from_slice(&points[source..source + dim]);
        counts[code] = 1.0;
        sums[code * dim..(code + 1) * dim]
            .copy_from_slice(&centroids[code * dim..(code + 1) * dim]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain Lloyd reference on the same initialization, for the fit gate.
    fn lloyd(points: &[f64], n: usize, dim: usize, k: usize, iters: usize, seed: u64) -> Vec<f64> {
        let mut centroids = init_centroids(points, n, dim, k, seed);
        for _ in 0..iters {
            let mut sums = vec![0.0f64; k * dim];
            let mut counts = vec![0usize; k];
            for index in 0..n {
                let point = &points[index * dim..(index + 1) * dim];
                let code = nearest(&centroids, dim, point);
                counts[code] += 1;
                for j in 0..dim {
                    sums[code * dim + j] += point[j];
                }
            }
            for code in 0..k {
                if counts[code] == 0 {
                    continue;
                }
                let inv = 1.0 / counts[code] as f64;
                for j in 0..dim {
                    centroids[code * dim + j] = sums[code * dim + j] * inv;
                }
            }
        }
        centroids
    }

    fn synthetic(k: usize, n: usize, dim: usize, seed: u64) -> Vec<f64> {
        let mut state = seed | 1;
        let centers: Vec<f64> = (0..k * dim)
            .map(|_| uniform(&mut state) * 10.0 - 5.0)
            .collect();
        let mut points = Vec::with_capacity(n * dim);
        for index in 0..n {
            let center = &centers[(index % k) * dim..(index % k + 1) * dim];
            for j in 0..dim {
                points.push(center[j] + (uniform(&mut state) - 0.5) * 2.0);
            }
        }
        points
    }

    #[test]
    fn e_means_reaches_lloyd_at_a_quarter_of_the_passes() {
        let (k, n, dim) = (16usize, 2048usize, 8usize);
        let points = synthetic(k, n, dim, 0xE3A5_0000_0001);
        let e = fit(&points, n, dim, k, EPOCHS, 7).expect("fit");
        let l = lloyd(&points, n, dim, k, 16, 7);
        let same = lloyd(&points, n, dim, k, EPOCHS, 7);
        let (e_mse, l_mse, same_mse) = (
            mse(&points, n, dim, &e),
            mse(&points, n, dim, &l),
            mse(&points, n, dim, &same),
        );
        eprintln!(
            "e-means {EPOCHS} epochs vs Lloyd 16 passes: MSE {e_mse:.6} vs {l_mse:.6} ({:.3}x)",
            e_mse / l_mse
        );
        eprintln!(
            "e-means {EPOCHS} epochs vs Lloyd {EPOCHS} passes: MSE {e_mse:.6} vs {same_mse:.6} ({:.3}x)",
            e_mse / same_mse
        );
        assert!(
            e_mse <= l_mse * 1.10,
            "e-means is more than 10% worse than Lloyd"
        );
        assert_eq!(
            fit(&points, n, dim, k, EPOCHS, 7).expect("fit again"),
            e,
            "e-means is not deterministic for a seed"
        );
    }

    #[test]
    fn a_bad_shape_is_refused() {
        assert!(fit(&[0.0; 8], 4, 2, 2, EPOCHS, 1).is_ok());
        assert!(fit(&[0.0; 7], 4, 2, 2, EPOCHS, 1).is_err());
        assert!(fit(&[], 0, 2, 2, EPOCHS, 1).is_err());
    }
}
