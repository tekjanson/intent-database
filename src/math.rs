/// Generalized math utilities for vectors and clustering.
/// These are intentionally small, dependency-free helpers focused on
/// numerical stability for small-to-medium sized vectors used by the
/// funnel builder and tests.

/// Dot product of two same-length slices.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// L2 norm (Euclidean) of a vector.
pub fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Try to normalize a vector in-place. Returns true if normalization succeeded
/// (norm > eps), otherwise leaves vector unchanged and returns false.
pub fn normalize_inplace(v: &mut [f32], eps: f32) -> bool {
    let sum_sq: f32 = v.iter().map(|x| x * x).sum();
    if sum_sq <= eps {
        return false;
    }
    let norm = sum_sq.sqrt();
    for x in v.iter_mut() {
        *x /= norm;
    }
    true
}

/// Return a unit vector copy of the input, or a zero-vector if the input
/// is (near-)zero. Uses eps for stability.
pub fn unit_vector_or_zero(v: &[f32], eps: f32) -> Vec<f32> {
    let mut out = v.to_vec();
    if normalize_inplace(&mut out, eps) {
        out
    } else {
        for x in out.iter_mut() {
            *x = 0.0;
        }
        out
    }
}

/// Cosine similarity, robust to zero vectors. Returns 1.0 if either vector is zero.
pub fn cosine_similarity(a: &[f32], b: &[f32], eps: f32) -> f32 {
    let na = l2_norm(a);
    let nb = l2_norm(b);
    if na <= eps || nb <= eps {
        return 0.0; // treat as orthogonal to avoid NaNs
    }
    dot(a, b) / (na * nb)
}

/// Cosine distance (1 - cosine_similarity)
pub fn cosine_distance(a: &[f32], b: &[f32], eps: f32) -> f32 {
    1.0 - cosine_similarity(a, b, eps)
}

/// Compute centroid (mean) of a non-empty set of points and return a unit-length
/// vector. If points is empty, returns an empty Vec. If centroid is zero-length,
/// returns a zero-vector of the same dimension.
pub fn centroid(points: &[Vec<f32>], eps: f32) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }
    let dim = points[0].len();
    let mut acc = vec![0.0f32; dim];
    for p in points {
        for (i, v) in p.iter().enumerate() {
            acc[i] += *v;
        }
    }
    for x in acc.iter_mut() {
        *x /= points.len() as f32;
    }
    // normalize safely
    if normalize_inplace(&mut acc, eps) {
        acc
    } else {
        vec![0.0f32; dim]
    }
}

/// Compute inertia (sum of squared distances to respective centroids) for a set of
/// points and cluster assignments. Uses cosine distance as the point-centroid metric.
pub fn inertia(points: &[Vec<f32>], labels: &[usize], centroids: &[Vec<f32>], eps: f32) -> f32 {
    let mut total = 0.0f32;
    for (i, p) in points.iter().enumerate() {
        let lab = labels[i];
        if lab >= centroids.len() {
            continue;
        }
        let d = cosine_distance(p, &centroids[lab], eps);
        total += d * d;
    }
    total
}

/// Silhouette score for clustering using cosine distance. Returns average silhouette
/// across points. For small clusters (size 1) the silhouette is defined as 0.
pub fn silhouette_score(points: &[Vec<f32>], labels: &[usize], eps: f32) -> f32 {
    let n = points.len();
    if n == 0 {
        return 0.0;
    }
    // build cluster map
    use std::collections::HashMap;
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, &lab) in labels.iter().enumerate() {
        clusters.entry(lab).or_default().push(i);
    }

    let mut scores: Vec<f32> = Vec::with_capacity(n);
    for i in 0..n {
        let lab = labels[i];
        let a = if let Some(members) = clusters.get(&lab) {
            if members.len() <= 1 {
                0.0
            } else {
                // average distance to other points in same cluster
                let mut s = 0.0f32;
                for &j in members.iter() {
                    if j == i {
                        continue;
                    }
                    s += cosine_distance(&points[i], &points[j], eps);
                }
                s / ((members.len() - 1) as f32)
            }
        } else {
            0.0
        };

        // compute smallest average distance to other clusters
        let mut b = f32::MAX;
        for (other_lab, members) in clusters.iter() {
            if *other_lab == lab {
                continue;
            }
            let mut s = 0.0f32;
            for &j in members.iter() {
                s += cosine_distance(&points[i], &points[j], eps);
            }
            let avg = s / (members.len() as f32);
            if avg < b {
                b = avg;
            }
        }
        let s_i = if b.is_finite() {
            if a < b {
                1.0 - (a / b)
            } else {
                (b / a) - 1.0
            }
        } else {
            0.0
        };
        scores.push(s_i);
    }

    scores.iter().sum::<f32>() / (scores.len() as f32)
}

/// Compute the dominant principal component (unit vector) using power iteration.
/// Returns a unit vector of dimension == points[0].len(). If power iteration
/// fails (zero data), returns zero-vector.
pub fn principal_component(points: &[Vec<f32>], iters: usize, eps: f32) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }
    let dim = points[0].len();
    // start with a random-ish vector derived deterministically from data
    let mut v = vec![0.0f32; dim];
    for (i, x) in v.iter_mut().enumerate().take(dim) {
        *x = i as f32 + 1.0;
    }
    if !normalize_inplace(&mut v, eps) {
        return vec![0.0f32; dim];
    }

    for _ in 0..iters {
        // multiply covariance-like operator: w = sum_p (p dot v) * p
        let mut w = vec![0.0f32; dim];
        for p in points.iter() {
            let alpha = dot(p, &v);
            for i in 0..dim {
                w[i] += alpha * p[i];
            }
        }
        if !normalize_inplace(&mut w, eps) {
            break;
        }
        // check convergence
        let mv = 1.0 - dot(&w, &v).abs();
        v = w;
        if mv <= eps {
            break;
        }
    }
    v
}
