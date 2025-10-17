use crate::math;

pub fn spherical_kmeans(
    data: &[Vec<f32>],
    k: usize,
    max_iters: usize,
    tol: f32,
    seed: Option<u64>,
) -> Vec<Vec<f32>> {
    let n = data.len();
    if k == 0 || n == 0 {
        return Vec::new();
    }

    let mut data_n: Vec<Vec<f32>> = data.to_owned();
    for v in data_n.iter_mut() {
        let _ = math::normalize_inplace(v, 1e-12);
    }

    let mut indices: Vec<usize> = (0..n).collect();
    if let Some(s) = seed {
        let mut rnd = s.wrapping_add(0x9e3779b97f4a7c15u64);
        for i in (1..n).rev() {
            rnd = rnd.wrapping_mul(6364136223846793005u64).wrapping_add(1442695040888963407u64);
            let j = (rnd as usize) % (i + 1);
            indices.swap(i, j);
        }
    }

    let mut centroids: Vec<Vec<f32>> = Vec::new();
    centroids.push(data_n[indices[0]].clone());
    while centroids.len() < k {
        let mut best_idx = indices[0];
        let mut best_min = -1.0f32;
        for &i in indices.iter() {
            let p = &data_n[i];
            if centroids.iter().any(|c| p.iter().zip(c.iter()).all(|(x, y)| (x - y).abs() < 1e-9)) {
                continue;
            }
            let mut min_d = f32::MAX;
            for c in centroids.iter() {
                let d = math::cosine_distance(p, c, 1e-12);
                if d < min_d {
                    min_d = d;
                }
            }
            if min_d > best_min {
                best_min = min_d;
                best_idx = i;
            }
        }
        centroids.push(data_n[best_idx].clone());
    }

    for _iter in 0..max_iters {
        let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); k];
        for (pi, p) in data_n.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                let d = math::cosine_distance(p, c, 1e-12);
                if d < best_d {
                    best_d = d;
                    best = ci;
                }
            }
            clusters[best].push(pi);
        }

        let mut max_move = 0.0f32;
        let old_centroids = centroids.clone();
        for ci in 0..k {
            if clusters[ci].is_empty() {
                let mut best_idx = 0usize;
                let mut best_min = -1.0f32;
                for (i, p) in data_n.iter().enumerate() {
                    if centroids
                        .iter()
                        .any(|c| p.iter().zip(c.iter()).all(|(x, y)| (x - y).abs() < 1e-9))
                    {
                        continue;
                    }
                    let mut min_d = f32::MAX;
                    for c in centroids.iter() {
                        let d = math::cosine_distance(p, c, 1e-12);
                        if d < min_d {
                            min_d = d;
                        }
                    }
                    if min_d > best_min {
                        best_min = min_d;
                        best_idx = i;
                    }
                }
                centroids[ci].clone_from(&data_n[best_idx]);
            } else {
                let points: Vec<Vec<f32>> =
                    clusters[ci].iter().map(|&pi| data_n[pi].clone()).collect();
                centroids[ci] = math::centroid(&points, 1e-12);
            }
            let mv = math::cosine_distance(&old_centroids[ci], &centroids[ci], 1e-12);
            if mv > max_move {
                max_move = mv;
            }
        }

        if max_move <= tol {
            break;
        }
    }
    centroids
}

pub fn select_k_by_silhouette(
    data: &[Vec<f32>],
    max_k: usize,
    max_iters: usize,
    seed: Option<u64>,
) -> usize {
    let n = data.len();
    if n == 0 {
        return 0;
    }
    let mut best_k = 1usize;
    let mut best_score = f32::MIN;
    for k in 1..=max_k {
        if k > n {
            break;
        }
        let cents = spherical_kmeans(data, k, max_iters, 1e-4, seed);
        let mut labels = vec![0usize; n];
        for (i, p) in data.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::MAX;
            for (ci, c) in cents.iter().enumerate() {
                let d = math::cosine_distance(p, c, 1e-12);
                if d < best_d {
                    best_d = d;
                    best = ci;
                }
            }
            labels[i] = best;
        }
        let score = crate::math::silhouette_score(data, &labels, 1e-12);
        if score > best_score || (score == best_score && k < best_k) {
            best_score = score;
            best_k = k;
        }
    }
    best_k
}

#[allow(dead_code)]
pub fn select_k_by_silhouette_wrapper(
    data: &[Vec<f32>],
    max_k: usize,
    max_iters: usize,
    seed: Option<u64>,
) -> usize {
    select_k_by_silhouette(data, max_k, max_iters, seed)
}

pub fn select_k_by_elbow(
    data: &[Vec<f32>],
    max_k: usize,
    max_iters: usize,
    seed: Option<u64>,
) -> usize {
    let n = data.len();
    if n == 0 {
        return 0;
    }
    let mut inertias: Vec<f32> = Vec::new();
    let mut ks: Vec<usize> = Vec::new();
    let mut silhouettes: Vec<f32> = Vec::new();
    let mut min_cluster_sizes: Vec<usize> = Vec::new();
    for k in 1..=max_k {
        if k > n {
            break;
        }
        let cents = spherical_kmeans(data, k, max_iters, 1e-4, seed);
        let mut labels = vec![0usize; n];
        for (i, p) in data.iter().enumerate() {
            let mut best = 0usize;
            let mut best_d = f32::MAX;
            for (ci, c) in cents.iter().enumerate() {
                let d = math::cosine_distance(p, c, 1e-12);
                if d < best_d {
                    best_d = d;
                    best = ci;
                }
            }
            labels[i] = best;
        }
        let inert = crate::math::inertia(data, &labels, &cents, 1e-12);
        inertias.push(inert);
        let sil = crate::math::silhouette_score(data, &labels, 1e-12);
        silhouettes.push(sil);
        ks.push(k);
        let mut counts = vec![0usize; k];
        for &lab in labels.iter() {
            counts[lab] += 1;
        }
        let min_sz = *counts.iter().min().unwrap_or(&0usize);
        min_cluster_sizes.push(min_sz);
    }
    if inertias.len() <= 1 {
        return ks[0];
    }
    let mut best_sil = f32::MIN;
    let mut best_sil_k = ks[0];
    for (i, &k) in ks.iter().enumerate() {
        if min_cluster_sizes[i] < 2 {
            continue;
        }
        let s = silhouettes[i];
        if s > best_sil || (s == best_sil && k < best_sil_k) {
            best_sil = s;
            best_sil_k = k;
        }
    }
    if best_sil > 1e-3 {
        return best_sil_k;
    }
    let m = inertias.len();
    let x0 = 0f32;
    let y0 = inertias[0];
    let x1 = (m - 1) as f32;
    let y1 = inertias[m - 1];
    let mut best_idx: usize = 0;
    let mut best_dist = f32::MIN;
    let dx = x1 - x0;
    let dy = y1 - y0;
    let denom = (dx * dx + dy * dy).sqrt();
    if denom <= 1e-12 {
        return ks[0];
    }
    for (i, &y) in inertias.iter().enumerate() {
        let xi = i as f32;
        let num = (dy * xi - dx * y + x1 * y0 - y1 * x0).abs();
        let dist = num / denom;
        if dist > best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    let selected = ks[best_idx];
    if selected == *ks.last().unwrap() && best_idx > 0 {
        return ks[best_idx - 1];
    }
    selected
}
