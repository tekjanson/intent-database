use intent_database::storage::{SledStorage, Storage};
use tempfile::tempdir;

#[tokio::test]
async fn test_sled_put_get_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sled_db");
    let s = SledStorage::new(path.clone());

    s.put_blob("k", b"v").await.unwrap();
    let got = s.get_blob("k").await.unwrap();
    assert_eq!(got.unwrap(), b"v".to_vec());

    // reopen using a new adapter instance
    let s2 = SledStorage::new(path);
    let got2 = s2.get_blob("k").await.unwrap();
    assert_eq!(got2.unwrap(), b"v".to_vec());
}
