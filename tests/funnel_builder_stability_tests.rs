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
async fn test_small_tol_converges() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    let embed = Arc::new(HashEmbedder::new(32));
    let mut convs = Vec::new();
    for i in 0..20 {
        convs.push(make_conversation(&format!("s{}", i), &format!("pt{}", i)));
    }

    let mut builder = ClusterFunnelBuilder::new(4, 500);
    builder.tol = 1e-8;
    let funnel = builder
        .build_and_persist("st", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert!(!funnel.node_ids.is_empty());
}
