use intent_database::funnel::HashEmbedder;
use intent_database::Embedder;

#[tokio::test]
async fn test_spherical_kmeans_converges_and_centroids_normalized() {
    // create three well-separated pseudo-embeddings using HashEmbedder on different texts
    let h = HashEmbedder::new(8);
    let a = h.embed("alpha").await.unwrap();
    let b = h.embed("bravo").await.unwrap();
    let c = h.embed("charlie").await.unwrap();
    let data = vec![a.clone(), b.clone(), c.clone(), a.clone(), b.clone(), c.clone()];

    let centroids = intent_database::funnel_builder::spherical_kmeans(&data, 3, 100, 1e-6, None);
    assert_eq!(centroids.len(), 3);
    for cent in centroids.iter() {
        // centroid should be normalized
        let sum_sq: f32 = cent.iter().map(|x| x * x).sum();
        let norm = sum_sq.sqrt();
        let diff = (norm - 1.0).abs();
        assert!(diff < 1e-3, "centroid not normalized: {}", norm);
    }
}

#[tokio::test]
async fn test_empty_cluster_reseeding_behavior() {
    let h = HashEmbedder::new(8);
    // two identical points and one distant point; request k=3 which forces an empty cluster
    let p1 = h.embed("same").await.unwrap();
    let p2 = p1.clone();
    let p3 = h.embed("far_away").await.unwrap();
    let data = vec![p1, p2, p3];

    let centroids = intent_database::funnel_builder::spherical_kmeans(&data, 3, 50, 1e-6, None);
    assert_eq!(centroids.len(), 3);
    // ensure centroids are normalized and finite
    for c in centroids.iter() {
        assert!(c.iter().all(|v| v.is_finite()));
        let sum_sq: f32 = c.iter().map(|x| x * x).sum();
        let norm = sum_sq.sqrt();
        assert!((norm - 1.0).abs() < 1e-3);
    }
}

#[tokio::test]
async fn test_with_seed_produces_deterministic_shuffle() {
    let h = HashEmbedder::new(8);
    let mut data = Vec::new();
    for i in 0..12 {
        data.push(h.embed(&format!("pt{}", i)).await.unwrap());
    }
    let c1 = intent_database::funnel_builder::spherical_kmeans(&data, 4, 50, 1e-6, Some(42));
    let c2 = intent_database::funnel_builder::spherical_kmeans(&data, 4, 50, 1e-6, Some(42));
    assert_eq!(c1.len(), c2.len());
    for (a, b) in c1.iter().zip(c2.iter()) {
        for (x, y) in a.iter().zip(b.iter()) {
            assert!((x - y).abs() < 1e-6);
        }
    }
}
