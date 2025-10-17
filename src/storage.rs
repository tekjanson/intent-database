use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn put_blob(&self, key: &str, data: &[u8]) -> Result<(), String>;
    async fn get_blob(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
}

/// Sled-backed storage adapter (opens a single sled::Db per instance)
pub struct SledStorage {
    // Optional opened DB handle. If None, open will be attempted lazily.
    db: std::sync::Arc<std::sync::RwLock<Option<sled::Db>>>,
    path: PathBuf,
}

impl SledStorage {
    pub fn new(path: PathBuf) -> Self {
        static REGISTRY: Lazy<Mutex<HashMap<String, sled::Db>>> =
            Lazy::new(|| Mutex::new(HashMap::new()));
        let key = path.to_string_lossy().to_string();
        // First check registry
        if let Some(db) = REGISTRY.lock().unwrap().get(&key) {
            let db_clone = db.clone();
            let guard = std::sync::RwLock::new(Some(db_clone));
            return Self { db: std::sync::Arc::new(guard), path };
        }

        // Try to open now; if it fails, store None and allow methods to return errors.
        let db_opt = match sled::open(&path) {
            Ok(db) => {
                // insert into registry for future reuse
                REGISTRY.lock().unwrap().insert(key.clone(), db.clone());
                Some(db)
            }
            Err(_) => None,
        };
        Self { db: std::sync::Arc::new(std::sync::RwLock::new(db_opt)), path }
    }

    pub fn open_sync(&self) -> Option<std::sync::Arc<sled::Db>> {
        let guard = self.db.read().unwrap();
        guard.as_ref().map(|db| std::sync::Arc::new(db.clone()))
    }

    fn ensure_open(&self) -> Result<std::sync::Arc<sled::Db>, String> {
        // Fast path: check read lock
        {
            let guard = self.db.read().unwrap();
            if let Some(db) = guard.as_ref() {
                return Ok(std::sync::Arc::new(db.clone()));
            }
        }

        // Try to open and write it
        let path = self.path.clone();
        match sled::open(path) {
            Ok(db) => {
                let mut guard = self.db.write().unwrap();
                *guard = Some(db.clone());
                Ok(std::sync::Arc::new(db))
            }
            Err(e) => Err(format!("failed to open sled db: {}", e)),
        }
    }
}

// InMemoryStorage intentionally removed to enforce single-canonical storage (sled)

#[async_trait]
impl Storage for SledStorage {
    async fn put_blob(&self, key: &str, data: &[u8]) -> Result<(), String> {
        let key = key.to_string();
        let data = data.to_vec();
        // attempt to ensure DB is open (may return Err)
        let db = self.ensure_open()?;
        let res: Result<(), String> = tokio::task::spawn_blocking(move || -> Result<(), String> {
            db.insert(key.as_bytes(), data).map_err(|e| e.to_string())?;
            db.flush().map_err(|e| e.to_string())?;
            Ok(())
        })
        .await
        .map_err(|e| e.to_string())?;

        res
    }

    async fn get_blob(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let key = key.to_string();
        let db = self.ensure_open()?;
        let res: Result<Option<Vec<u8>>, String> =
            tokio::task::spawn_blocking(move || -> Result<Option<Vec<u8>>, String> {
                match db.get(key.as_bytes()) {
                    Ok(Some(v)) => Ok(Some(v.to_vec())),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await
            .map_err(|e| e.to_string())?;

        res
    }
}
