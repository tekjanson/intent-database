use intent_database::funnel::HashEmbedder;
use intent_database::funnel::ModelAdapter;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::Storage;
use intent_database::Conversation;
use intent_database::SledStorage;
use intent_database::TemplateModelAdapter;
use std::sync::Arc;
use tempfile::TempDir;

fn make_conversation(id: &str, purpose: &str) -> Conversation {
    let intent = intent_database::Intent::new(purpose.to_string());
    Conversation::new(id.to_string(), intent, intent_database::Sentiment::Neutral)
}

#[tokio::test]
async fn test_build_then_replay_and_append_conversation() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    // Create some conversations and build a funnel
    let convs = vec![
        make_conversation("r1", "billing"),
        make_conversation("r2", "signup"),
        make_conversation("r3", "billing plan"),
    ];
    let embed = Arc::new(HashEmbedder::new(16));
    let builder = ClusterFunnelBuilder::new(2, 50);
    let _funnel = builder
        .build_and_persist("replay-f", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");

    // Persist a conversation manually as the system would on thumbs-up
    let conv = make_conversation("rc1", "billing followup");
    let bytes = bincode::serialize(&conv).unwrap();
    let key = format!("conv:{}", conv.id);
    storage.put_blob(&key, &bytes).await.expect("put blob");

    // Simulate replay: load conversation from storage, generate a follow-up question,
    // append it to the conversation entries and re-persist.
    let got = storage.get_blob(&key).await.expect("get blob");
    assert!(got.is_some());
    let mut conv_loaded: Conversation = bincode::deserialize(&got.unwrap()).unwrap();

    let model = TemplateModelAdapter::new();
    let q = model.generate_question(&conv_loaded.intent.purpose).await.expect("gen q");
    // append a synthetic assistant entry
    conv_loaded.add_entry(intent_database::ConversationEntry::new("assistant".to_string(), q));

    // re-persist
    let new_bytes = bincode::serialize(&conv_loaded).unwrap();
    storage.put_blob(&key, &new_bytes).await.expect("put blob 2");

    // verify blob updated
    let got2 = storage.get_blob(&key).await.expect("get blob 2");
    let conv2: Conversation = bincode::deserialize(&got2.unwrap()).unwrap();
    assert_eq!(conv2.entries.len(), 1);
}
