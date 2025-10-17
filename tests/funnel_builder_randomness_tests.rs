use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::spherical_kmeans;
use intent_database::Embedder;

#[tokio::test]
async fn test_seed_vs_no_seed_differs() {
    let h = HashEmbedder::new(32);
    let mut data = Vec::new();
    for i in 0..30 {
        data.push(h.embed(&format!("pt{}", i)).await.unwrap());
    }
    let c_seed = spherical_kmeans(&data, 5, 100, 1e-6, Some(7));
    let c_noseed = spherical_kmeans(&data, 5, 100, 1e-6, None);
    // They should not be exactly equal; check sum absolute diff across centroids
    let mut total_diff = 0.0f32;
    let m = std::cmp::min(c_seed.len(), c_noseed.len());
    for i in 0..m {
        for (a, b) in c_seed[i].iter().zip(c_noseed[i].iter()) {
            total_diff += (a - b).abs();
        }
    }
    assert!(total_diff > 1e-6, "seeded and non-seeded centroids are too similar: {}", total_diff);
}
