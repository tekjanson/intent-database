use intent_database::funnel::{HashEmbedder, StubEmbedder};
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::Storage;
use intent_database::Conversation;
use intent_database::SledStorage;

use std::sync::Arc;
use tempfile::TempDir;

fn make_conversation(id: &str, purpose: &str) -> Conversation {
    let intent = intent_database::Intent::new(purpose.to_string());
    Conversation::new(id.to_string(), intent, intent_database::Sentiment::Neutral)
}

#[tokio::test]
async fn test_cluster_builder_persists_funnel_and_nodes_end_to_end() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let storage = Arc::new(storage);

    let embed = Arc::new(HashEmbedder::new(8));
    let convs = vec![
        make_conversation("c1", "payment issue"),
        make_conversation("c2", "billing question"),
        make_conversation("c3", "signup problem"),
    ];

    let builder = ClusterFunnelBuilder::new(2, 50);
    let funnel = builder
        .build_and_persist("testf", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert!(!funnel.node_ids.is_empty());

    // ensure funnel blob exists in storage by trying to load the key
    let key = format!("funnel:builder:{}", "testf");
    let _ = storage.get_blob(&key).await.expect("get blob");
}

#[tokio::test]
async fn test_deterministic_seed_repro_when_persisted() {
    let tmp = TempDir::new().unwrap();
    let storage1 = Arc::new(SledStorage::new(tmp.path().to_path_buf()));
    let storage2 = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    let embed = Arc::new(HashEmbedder::new(8));
    let convs = vec![
        make_conversation("c1", "alpha"),
        make_conversation("c2", "bravo"),
        make_conversation("c3", "charlie"),
        make_conversation("c4", "delta"),
    ];

    let mut b1 = ClusterFunnelBuilder::with_seed(3, 50, 12345);
    let mut b2 = ClusterFunnelBuilder::with_seed(3, 50, 12345);
    b1.tol = 1e-6;
    b2.tol = 1e-6;

    let f1 =
        b1.build_and_persist("f1", &convs, embed.clone(), storage1.clone()).await.expect("build1");
    let f2 =
        b2.build_and_persist("f2", &convs, embed.clone(), storage2.clone()).await.expect("build2");

    // funnels should have same number of nodes and nodes should be deterministically assigned
    assert_eq!(f1.node_ids.len(), f2.node_ids.len());
}

#[tokio::test]
async fn test_zero_vector_embedder_handling() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));
    let embed = Arc::new(StubEmbedder {});

    let convs = vec![make_conversation("c1", "x"), make_conversation("c2", "y")];
    let builder = ClusterFunnelBuilder::new(2, 20);

    // Should not panic and should produce normalized centroids (or valid centroids)
    let funnel = builder
        .build_and_persist("zf", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert!(!funnel.node_ids.is_empty());
}

#[tokio::test]
async fn test_concurrent_builds_same_storage() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));
    let embed = Arc::new(HashEmbedder::new(8));

    let convs_a = vec![make_conversation("a1", "one"), make_conversation("a2", "two")];
    let convs_b = vec![make_conversation("b1", "three"), make_conversation("b2", "four")];

    let s1 = storage.clone();
    let s2 = storage.clone();
    let e1 = embed.clone();
    let e2 = embed.clone();

    let t1 = tokio::spawn(async move {
        let b = ClusterFunnelBuilder::new(2, 40);
        b.build_and_persist("con1", &convs_a, e1, s1).await
    });
    let t2 = tokio::spawn(async move {
        let b = ClusterFunnelBuilder::new(2, 40);
        b.build_and_persist("con2", &convs_b, e2, s2).await
    });

    let r1 = t1.await.unwrap().expect("t1");
    let r2 = t2.await.unwrap().expect("t2");
    assert!(!r1.node_ids.is_empty() && !r2.node_ids.is_empty());
}
