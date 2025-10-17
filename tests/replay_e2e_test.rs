use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::SledStorage;
use intent_database::storage::Storage;
use intent_database::TemplateModelAdapter;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_replay_and_append() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_replay");
    let storage = Arc::new(SledStorage::new(path));

    let conv =
        Conversation::new("c1".to_string(), Intent::new("hello".to_string()), Sentiment::Neutral);
    let embedder = Arc::new(intent_database::funnel::HashEmbedder::new(8));
    let builder = ClusterFunnelBuilder::new(1, 3);

    let funnel = builder
        .build_and_persist("r1", &[conv.clone()], embedder.clone(), storage.clone())
        .await
        .expect("build");
    // simulate loading and replay: activate funnel and generate question
    let model = Arc::new(TemplateModelAdapter::new());
    let funnel_arc = Arc::new(funnel);
    let mut session = funnel_arc.activate(embedder.clone(), model.clone()).await;
    if let Some(q) = session.next_question() {
        // append a follow-up conversation and persist
        let follow = Conversation::new("c2".to_string(), Intent::new(q.prompt), Sentiment::Neutral);
        let bytes = bincode::serialize(&follow).unwrap();
        storage.put_blob("conv:c2", &bytes).await.unwrap();
    }
    // verify the persisted conv exists
    let got = storage.get_blob("conv:c2").await.unwrap();
    assert!(got.is_some());
}
