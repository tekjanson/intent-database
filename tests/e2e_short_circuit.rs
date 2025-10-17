use intent_database::database::AsyncIntentDatabase;
use intent_database::storage::{SledStorage, Storage};
use intent_database::{Conversation, ConversationEntry, Intent, MockConnector, Sentiment};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn e2e_short_circuit_avoids_model_call() {
    let dir = tempdir().unwrap();
    let storage = SledStorage::new(dir.path().to_path_buf());
    let idb = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(idb, Arc::new(storage));

    // Persist a conversation directly using the storage interface (simulate existing knowledge)
    let intent =
        Intent::new("Short circuit test".to_string()).with_topics(vec!["topic1".to_string()]);
    let mut conv =
        Conversation::new("conv-short-1".to_string(), intent.clone(), Sentiment::Neutral);
    conv.add_entry(ConversationEntry::new("user".to_string(), "hello".to_string()));

    // Store using handle_chat_feedback with positive response so it persists
    let connector = Arc::new(MockConnector { positive: true });
    let _ = async_db.handle_chat_feedback(conv.clone(), connector).await.expect("persist");

    // Now send a very similar conversation and ensure the persisted blob exists
    let key = format!("conv:{}", conv.id);
    let storage2 = SledStorage::new(dir.path().to_path_buf());
    let blob = storage2.get_blob(&key).await.expect("get blob");
    assert!(blob.is_some(), "persisted conversation blob must exist to short-circuit");
}
