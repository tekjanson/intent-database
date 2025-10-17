use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::SledStorage;
use intent_database::storage::Storage;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_cluster_builder_persists_nodes() {
    // create simple conversations with two obvious clusters by purpose
    let convs = vec![
        Conversation::new(
            "a1".to_string(),
            Intent::new("billing issue".to_string()),
            Sentiment::Neutral,
        ),
        Conversation::new(
            "a2".to_string(),
            Intent::new("billing question".to_string()),
            Sentiment::Neutral,
        ),
        Conversation::new(
            "b1".to_string(),
            Intent::new("login problem".to_string()),
            Sentiment::Neutral,
        ),
        Conversation::new(
            "b2".to_string(),
            Intent::new("cannot login".to_string()),
            Sentiment::Neutral,
        ),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_cluster_funnel");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(2, 5);
    let embedder = Arc::new(HashEmbedder::new(16));

    let funnel = builder.build_and_persist("c1", &convs, embedder, storage.clone()).await.unwrap();
    assert!(!funnel.node_ids.is_empty());

    // verify nodes persisted
    for nid in funnel.node_ids.iter() {
        let key = format!("funnel:node:{}", nid);
        let blob = storage.get_blob(&key).await.unwrap();
        assert!(blob.is_some());
    }
}

#[tokio::test]
async fn test_cluster_builder_handles_empty_input() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_cluster_funnel_empty");
    let storage = Arc::new(SledStorage::new(path));

    let builder = ClusterFunnelBuilder::new(3, 3);
    let embedder = Arc::new(HashEmbedder::new(16));

    let convs: Vec<Conversation> = Vec::new();
    let funnel =
        builder.build_and_persist("c_empty", &convs, embedder, storage.clone()).await.unwrap();
    assert!(funnel.node_ids.is_empty());
}
