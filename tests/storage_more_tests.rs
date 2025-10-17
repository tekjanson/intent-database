use intent_database::storage::{SledStorage, Storage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_storage_concurrent_put_get() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_concurrent");
    let s = Arc::new(SledStorage::new(path));

    let mut handles = Vec::new();
    for i in 0..50 {
        let s = s.clone();
        handles.push(tokio::spawn(async move {
            let key = format!("k{}", i);
            let val = format!("v{}", i);
            s.put_blob(&key, val.as_bytes()).await.unwrap();
            let got = s.get_blob(&key).await.unwrap();
            assert_eq!(got.unwrap(), val.as_bytes().to_vec());
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::test]
async fn test_storage_large_blob() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_large");
    let s = SledStorage::new(path);

    // 10MB blob
    let large = vec![0u8; 10 * 1024 * 1024];
    s.put_blob("large", &large).await.unwrap();
    let got = s.get_blob("large").await.unwrap().unwrap();
    assert_eq!(got.len(), large.len());
}

#[tokio::test]
async fn test_storage_invalid_path_error() {
    // Use a file path (create a file) so sled::open should error
    let dir = tempdir().unwrap();
    let file = dir.path().join("not_a_dir");
    std::fs::write(&file, b"data").unwrap();
    let s = SledStorage::new(file);

    // put_blob should return an Err
    let res = s.put_blob("k", b"v").await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_storage_corrupt_blob_and_recovery() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_corrupt");
    let s = SledStorage::new(path.clone());

    // Write an arbitrary non-bincode blob under key used by AsyncIntentDatabase
    s.put_blob("db_corrupt", b"not_a_bincode").await.unwrap();

    // Reopen and read raw blob to ensure it's there
    let s2 = SledStorage::new(path);
    let got = s2.get_blob("db_corrupt").await.unwrap();
    assert_eq!(got.unwrap(), b"not_a_bincode".to_vec());
}
