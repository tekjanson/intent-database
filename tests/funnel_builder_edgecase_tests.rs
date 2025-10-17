use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::Conversation;
use intent_database::SledStorage;

use std::sync::Arc;
use tempfile::TempDir;

fn make_conversation(id: &str, purpose: &str) -> Conversation {
    let intent = intent_database::Intent::new(purpose.to_string());
    Conversation::new(id.to_string(), intent, intent_database::Sentiment::Neutral)
}

#[tokio::test]
async fn test_high_dimensional_vectors() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    let embed = Arc::new(HashEmbedder::new(128));
    let mut convs = Vec::new();
    for i in 0..50 {
        convs.push(make_conversation(&format!("c{}", i), &format!("pt{}", i)));
    }

    let builder = ClusterFunnelBuilder::new(5, 100);
    let funnel = builder
        .build_and_persist("hd", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert_eq!(funnel.node_ids.len(), std::cmp::min(5, convs.len()));
}

#[tokio::test]
async fn test_large_k_relative_to_n() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    let embed = Arc::new(HashEmbedder::new(16));
    let convs = vec![
        make_conversation("c1", "one"),
        make_conversation("c2", "two"),
        make_conversation("c3", "three"),
    ];

    let builder = ClusterFunnelBuilder::new(10, 50);
    let funnel = builder
        .build_and_persist("lk", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert_eq!(funnel.node_ids.len(), std::cmp::min(10, convs.len()));
}
