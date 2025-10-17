#![allow(clippy::useless_vec)]
use intent_database::math::*;

#[test]
fn test_cluster_quality_intra_vs_inter() {
    // create 3 tight clusters in 2D
    let a = vec![vec![1.0f32, 0.0], vec![0.9, 0.1], vec![0.95, 0.05]];
    let b = vec![vec![0.0f32, 1.0], vec![0.1, 0.9], vec![0.05, 0.95]];
    let mut pts = Vec::new();
    pts.extend(a.iter().cloned());
    pts.extend(b.iter().cloned());
    let labels = vec![0usize, 0usize, 0usize, 1usize, 1usize, 1usize];
    // compute centroids
    let _c0 = centroid(
        &pts.iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, p)| if labels[i] == 0 { Some(p) } else { None })
            .collect::<Vec<_>>(),
        1e-12,
    );
    let _c1 = centroid(
        &pts.iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, p)| if labels[i] == 1 { Some(p) } else { None })
            .collect::<Vec<_>>(),
        1e-12,
    );
    // compute average intra distance
    let mut intra = 0.0f32;
    let mut intra_count = 0usize;
    for i in 0..pts.len() {
        for j in (i + 1)..pts.len() {
            if labels[i] == labels[j] {
                intra += cosine_distance(&pts[i], &pts[j], 1e-12);
                intra_count += 1;
            }
        }
    }
    let intra_avg = intra / (intra_count as f32);
    // inter average
    let mut inter = 0.0f32;
    let mut inter_count = 0usize;
    for i in 0..pts.len() {
        for j in 0..pts.len() {
            if labels[i] != labels[j] {
                inter += cosine_distance(&pts[i], &pts[j], 1e-12);
                inter_count += 1;
            }
        }
    }
    let inter_avg = inter / (inter_count as f32);
    assert!(intra_avg < inter_avg);
}
