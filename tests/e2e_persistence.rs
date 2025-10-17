use intent_database::database::AsyncIntentDatabase;
use intent_database::storage::{SledStorage, Storage};
use intent_database::{Conversation, ConversationEntry, Intent, Sentiment};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn e2e_persistence_writes_blobs() {
    let dir = tempdir().unwrap();
    let storage = SledStorage::new(dir.path().to_path_buf());

    let idb = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(idb, Arc::new(storage));

    let intent = Intent::new("Test persist".to_string());
    let mut conv = Conversation::new("conv-e2e-1".to_string(), intent, Sentiment::Neutral);
    conv.add_entry(ConversationEntry::new("user".to_string(), "hello".to_string()));

    // Use a mock connector that returns thumbs-up so the AsyncIntentDatabase will persist it
    let connector = Arc::new(intent_database::MockConnector { positive: true });
    let ok = async_db.handle_chat_feedback(conv.clone(), connector).await.expect("send");
    assert!(ok, "feedback should be positive");

    // Verify the conversation blob exists in storage. Open a fresh SledStorage to check.
    let key = format!("conv:{}", conv.id);
    let storage2 = SledStorage::new(dir.path().to_path_buf());
    let blob = storage2.get_blob(&key).await.expect("get blob");
    assert!(blob.is_some(), "conversation blob should be present");
}
