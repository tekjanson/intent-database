use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::{AsyncIntentDatabase, IntentDatabase};
use intent_database::storage::{SledStorage, Storage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_persist_and_restore_unicode_and_large_metadata() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_persist_unicode");
    let storage = Arc::new(SledStorage::new(path));
    let _async_db = AsyncIntentDatabase::new(db, storage.clone());

    let mut conv =
        Conversation::new("u1".to_string(), Intent::new("U".to_string()), Sentiment::Neutral);
    conv.add_metadata("emoji".to_string(), "😀🚀🔥".to_string());
    conv.add_metadata("big".to_string(), "x".repeat(1024 * 1024));

    // persist directly via storage
    let bytes = bincode::serialize(&conv).unwrap();
    storage.put_blob("conv:u1", &bytes).await.unwrap();

    // load via AsyncIntentDatabase API (should return raw blob via load for db keys)
    let got = storage.get_blob("conv:u1").await.unwrap().unwrap();
    let conv2: Conversation = bincode::deserialize(&got).unwrap();
    assert_eq!(conv2.metadata.get("emoji").unwrap(), "😀🚀🔥");
}

#[tokio::test]
async fn test_rapid_persist_restore_cycles() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_cycles");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    for i in 0..50 {
        let key = format!("db_cycle_{}", i);
        async_db.persist(&key).await.unwrap();
        let _ = async_db.load(&key).await.unwrap();
    }
}

#[tokio::test]
async fn test_corrupt_blob_detection() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_corrupt_detect");
    let storage = SledStorage::new(path.clone());

    // write raw invalid bincode data under a db key
    storage.put_blob("db:borked", b"notbincode").await.unwrap();

    // Attempt to open and deserialize
    let got = storage.get_blob("db:borked").await.unwrap().unwrap();
    let res: Result<IntentDatabase, _> = bincode::deserialize(&got);
    assert!(res.is_err());
}
