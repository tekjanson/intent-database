use intent_database::database::{AsyncIntentDatabase, IntentDatabase};
use intent_database::storage::SledStorage;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_async_db_persist_load() {
    let db = IntentDatabase::new();
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_db_async");
    let storage = Arc::new(SledStorage::new(path));
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    async_db.persist("db1").await.unwrap();
    let loaded = async_db.load("db1").await.unwrap();
    assert!(loaded.is_some());
}
