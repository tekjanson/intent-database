use intent_database::math::*;

#[test]
fn test_dot_and_norm() {
    let a = vec![1.0f32, 2.0, 3.0];
    let b = vec![4.0f32, 5.0, 6.0];
    let d = dot(&a, &b);
    assert!((d - 32.0).abs() < 1e-6);
    let n = l2_norm(&a);
    assert!((n - (14.0f32).sqrt()).abs() < 1e-6);
}

#[test]
fn test_unit_vector_or_zero_and_normalize() {
    let mut v = vec![0.0f32, 0.0, 0.0];
    assert!(!normalize_inplace(&mut v, 1e-12));
    let u = unit_vector_or_zero(&v, 1e-12);
    assert!(u.iter().all(|x| *x == 0.0));

    let mut w = vec![3.0f32, 4.0];
    assert!(normalize_inplace(&mut w, 1e-12));
    assert!((l2_norm(&w) - 1.0).abs() < 1e-6);
}

#[test]
fn test_centroid_and_inertia() {
    let pts = vec![vec![1.0f32, 0.0], vec![0.0, 1.0]];
    let c = centroid(&pts, 1e-12);
    // centroid should be normalized unit vector of mean [0.5, 0.5]
    assert!((l2_norm(&c) - 1.0).abs() < 1e-6);
    let labels = vec![0usize, 0usize];
    let inert = inertia(&pts, &labels, &[c.clone()], 1e-12);
    assert!(inert >= 0.0);
}

#[test]
fn test_silhouette_basic() {
    // two clear clusters
    let a = [vec![1.0f32, 0.0], vec![0.9, 0.1]];
    let b = [vec![0.0f32, 1.0], vec![0.1, 0.9]];
    let mut pts = Vec::new();
    pts.extend(a.iter().cloned());
    pts.extend(b.iter().cloned());
    let labels = vec![0usize, 0usize, 1usize, 1usize];
    let score = silhouette_score(&pts, &labels, 1e-12);
    assert!(score > 0.0);
}

#[test]
fn test_principal_component_basic() {
    // points along x-axis
    let pts = vec![vec![1.0f32, 0.0], vec![2.0, 0.0], vec![3.0, 0.0]];
    let pc = principal_component(&pts, 100, 1e-12);
    // principal component should align with x axis (positive first component)
    assert!(pc[0] > 0.9);
}
