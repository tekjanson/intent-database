use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::funnel_builder::SimpleFunnelBuilder;
use intent_database::storage::SledStorage;
use intent_database::storage::Storage;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_simple_funnel_builder_groups_and_persists() {
    let convs = vec![
        Conversation::new(
            "a1".to_string(),
            Intent::new("T1".to_string()).with_topics(vec!["alpha".to_string()]),
            Sentiment::Neutral,
        ),
        Conversation::new(
            "a2".to_string(),
            Intent::new("T2".to_string()).with_topics(vec!["alpha".to_string()]),
            Sentiment::Neutral,
        ),
        Conversation::new(
            "b1".to_string(),
            Intent::new("T3".to_string()).with_topics(vec!["beta".to_string()]),
            Sentiment::Neutral,
        ),
        Conversation::new("m1".to_string(), Intent::new("T4".to_string()), Sentiment::Neutral),
    ];

    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_funnel_builder");
    let storage = Arc::new(SledStorage::new(path));

    let builder = SimpleFunnelBuilder::new();
    let _funnel = builder.build_and_persist("test1", &convs, storage.clone()).await.unwrap();

    // verify funnel persisted
    let blob = storage.get_blob("funnel:builder:test1").await.unwrap();
    assert!(blob.is_some());
}
