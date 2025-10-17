use intent_database::connector::MockConnector;
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::{AsyncIntentDatabase, IntentDatabase};
use intent_database::storage::{SledStorage, Storage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_handle_chat_feedback_negative() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_conn_neg");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    let intent = Intent::new("Test".to_string());
    let conv = Conversation::new("id1".to_string(), intent, Sentiment::Neutral);

    let connector = Arc::new(MockConnector { positive: false });
    let res = async_db.handle_chat_feedback(conv, connector).await.unwrap();
    assert!(!res);
    // nothing persisted under conv:id1
    let got = storage.get_blob("conv:id1").await.unwrap();
    assert!(got.is_none());
}

#[tokio::test]
async fn test_handle_chat_feedback_positive() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_conn_pos");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    let intent = Intent::new("Test".to_string());
    let conv = Conversation::new("id2".to_string(), intent, Sentiment::Neutral);

    let connector = Arc::new(MockConnector { positive: true });
    let res = async_db.handle_chat_feedback(conv.clone(), connector).await.unwrap();
    assert!(res);

    let got = storage.get_blob("conv:id2").await.unwrap();
    assert!(got.is_some());
    let funnel_blob = storage.get_blob("funnel:id2").await.unwrap();
    assert!(funnel_blob.is_some());
}
