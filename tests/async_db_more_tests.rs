use intent_database::database::{AsyncIntentDatabase, IntentDatabase};
use intent_database::storage::SledStorage;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_async_db_concurrent_persist() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_async_concurrent");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = Arc::new(AsyncIntentDatabase::new(db, storage.clone()));

    let mut handles = Vec::new();
    for i in 0..20 {
        let async_db = async_db.clone();
        let key = format!("db_{}", i);
        handles.push(tokio::spawn(async move {
            async_db.persist(&key).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::test]
async fn test_async_db_load_nonexistent() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_async_nonexistent");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    let loaded = async_db.load("no_such_key").await.unwrap();
    assert!(loaded.is_none());
}
