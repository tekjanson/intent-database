use async_trait::async_trait;
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::{SledStorage, Storage};
use std::sync::Arc;
use tempfile::tempdir;

// Simple embedder that returns the same vector for every input
struct IdenticalEmbedder {
    v: Vec<f32>,
}

#[async_trait]
impl intent_database::funnel::Embedder for IdenticalEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(self.v.clone())
    }
}

// Embedder that returns zero vectors
struct ZeroEmbedder {
    dim: usize,
}

#[async_trait]
impl intent_database::funnel::Embedder for ZeroEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.0f32; self.dim])
    }
}

// Storage that always fails on put_blob
struct FailingStorage {}

#[async_trait]
impl Storage for FailingStorage {
    async fn put_blob(&self, _key: &str, _data: &[u8]) -> Result<(), String> {
        Err("simulated storage failure".to_string())
    }
    async fn get_blob(&self, _key: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
}

#[tokio::test]
async fn test_k_greater_than_n() {
    let convs = vec![
        Conversation::new("a".to_string(), Intent::new("x".to_string()), Sentiment::Neutral),
        Conversation::new("b".to_string(), Intent::new("y".to_string()), Sentiment::Neutral),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_k_gt_n");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(5, 3);
    let embedder = Arc::new(HashEmbedder::new(8));

    let funnel = builder.build_and_persist("kgt", &convs, embedder, storage.clone()).await.unwrap();
    // since k > n, builder should reduce k to n
    assert_eq!(funnel.node_ids.len(), 2);
}

#[tokio::test]
async fn test_k_equals_one_and_centroid_norm() {
    let convs = vec![
        Conversation::new("a".to_string(), Intent::new("alpha".to_string()), Sentiment::Neutral),
        Conversation::new("b".to_string(), Intent::new("beta".to_string()), Sentiment::Neutral),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_k1");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(1, 5);
    let embedder = Arc::new(HashEmbedder::new(16));

    let funnel = builder.build_and_persist("k1", &convs, embedder, storage.clone()).await.unwrap();
    assert_eq!(funnel.node_ids.len(), 1);

    // load the node and verify centroid is normalized (length ~= 1)
    let nkey = format!("funnel:node:{}:n0", "k1");
    let blob = storage.get_blob(&nkey).await.unwrap().unwrap();
    let node: intent_database::funnel::FunnelNode = bincode::deserialize(&blob).unwrap();
    if let Some(c) = node.centroid {
        let norm_sq: f32 = c.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();
        assert!(norm > 0.0 && (norm - 1.0).abs() < 1e-3);
    } else {
        panic!("expected centroid");
    }
}

#[tokio::test]
async fn test_identical_embeddings_all_assigned() {
    let convs = vec![
        Conversation::new("a1".to_string(), Intent::new("one".to_string()), Sentiment::Neutral),
        Conversation::new("a2".to_string(), Intent::new("two".to_string()), Sentiment::Neutral),
        Conversation::new("a3".to_string(), Intent::new("three".to_string()), Sentiment::Neutral),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_identical");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(3, 3);
    let ident = IdenticalEmbedder { v: vec![1.0; 8] };
    let embedder = Arc::new(ident);

    let funnel =
        builder.build_and_persist("ident", &convs, embedder, storage.clone()).await.unwrap();
    // sum of all example_ids across nodes should equal number of conversations
    let mut total = 0usize;
    for nid in funnel.node_ids.iter() {
        let key = format!("funnel:node:{}", nid);
        let blob = storage.get_blob(&key).await.unwrap().unwrap();
        let node: intent_database::funnel::FunnelNode = bincode::deserialize(&blob).unwrap();
        total += node.example_ids.len();
    }
    assert_eq!(total, convs.len());
}

#[tokio::test]
async fn test_zero_vector_embedder_no_panic() {
    let convs =
        vec![Conversation::new("z1".to_string(), Intent::new("z".to_string()), Sentiment::Neutral)];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_zero");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(2, 3);
    let embedder = Arc::new(ZeroEmbedder { dim: 8 });

    // should not panic and should produce a funnel (possibly with zero centroids)
    let funnel =
        builder.build_and_persist("zero", &convs, embedder, storage.clone()).await.unwrap();
    assert!(!funnel.node_ids.is_empty());
}

#[tokio::test]
async fn test_storage_failure_propagates() {
    let convs =
        vec![Conversation::new("a".to_string(), Intent::new("a".to_string()), Sentiment::Neutral)];

    let storage = Arc::new(FailingStorage {});
    let builder = ClusterFunnelBuilder::new(2, 2);
    let embedder = Arc::new(HashEmbedder::new(8));

    let res = builder.build_and_persist("fail", &convs, embedder, storage).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_concurrent_builds_same_storage() {
    // use tokio's join! macro for concurrency
    let convs = vec![
        Conversation::new("a1".to_string(), Intent::new("alpha".to_string()), Sentiment::Neutral),
        Conversation::new("a2".to_string(), Intent::new("beta".to_string()), Sentiment::Neutral),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_concurrent");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(2, 3);
    let embedder = Arc::new(HashEmbedder::new(8));

    let s1 = storage.clone();
    let s2 = storage.clone();
    let e1 = embedder.clone();
    let e2 = embedder.clone();
    let b1 = builder;
    let b2 = ClusterFunnelBuilder::new(2, 3);

    let c1 = convs.clone();
    let c2 = convs.clone();
    let f1 = async move { b1.build_and_persist("con1", &c1, e1, s1).await };
    let f2 = async move { b2.build_and_persist("con2", &c2, e2, s2).await };

    let (r1, r2) = tokio::join!(f1, f2);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}
