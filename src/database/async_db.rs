use crate::conversation::Conversation;
use crate::matcher::SimilarityScore;
use crate::storage::Storage;
use std::sync::Arc;

/// Async wrapper around IntentDatabase for storage-backed operations
pub struct AsyncIntentDatabase {
    pub inner: Arc<std::sync::Mutex<crate::database::IntentDatabase>>,
    pub storage: Arc<dyn Storage>,
}

impl AsyncIntentDatabase {
    pub fn new(db: crate::database::IntentDatabase, storage: Arc<dyn Storage>) -> Self {
        Self { inner: Arc::new(std::sync::Mutex::new(db)), storage }
    }

    pub async fn persist(&self, key: &str) -> Result<(), String> {
        let key = key.to_string();
        let bytes = tokio::task::spawn_blocking({
            let inner = self.inner.clone();
            move || -> Result<Vec<u8>, String> {
                let guard = inner.lock().map_err(|e| format!("lock error: {:?}", e))?;
                bincode::serialize(&*guard).map_err(|e| e.to_string())
            }
        })
        .await
        .map_err(|e| e.to_string())??;

        self.storage.put_blob(&key, &bytes).await
    }

    pub async fn load(&self, key: &str) -> Result<Option<crate::database::IntentDatabase>, String> {
        match self.storage.get_blob(key).await? {
            Some(bytes) => {
                let deserial = move || -> Result<crate::database::IntentDatabase, String> {
                    bincode::deserialize::<crate::database::IntentDatabase>(&bytes)
                        .map_err(|e| e.to_string())
                };

                let db =
                    tokio::task::spawn_blocking(deserial).await.map_err(|e| e.to_string())??;
                Ok(Some(db))
            }
            None => Ok(None),
        }
    }

    pub async fn get_best_match(
        &self,
        query: &Conversation,
    ) -> Result<Option<(String, crate::matcher::SimilarityScore)>, String> {
        let query = query.clone();
        let res = tokio::task::spawn_blocking({
            let inner = self.inner.clone();
            move || -> Result<Option<(String, SimilarityScore)>, String> {
                let guard = inner.lock().map_err(|e| format!("lock error: {:?}", e))?;
                Ok(guard.get_best_match(&query))
            }
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(res)
    }

    pub async fn store_local(&self, conversation: Conversation) -> Result<(), String> {
        let conversation = conversation.clone();
        tokio::task::spawn_blocking({
            let inner = self.inner.clone();
            move || -> Result<(), String> {
                let mut guard = inner.lock().map_err(|e| format!("lock error: {:?}", e))?;
                guard.store(conversation)
            }
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub async fn get_conversation_by_id(&self, id: &str) -> Result<Option<Conversation>, String> {
        let id = id.to_string();
        let res = tokio::task::spawn_blocking({
            let inner = self.inner.clone();
            move || -> Result<Option<Conversation>, String> {
                let guard = inner.lock().map_err(|e| format!("lock error: {:?}", e))?;
                Ok(guard.get(&id).cloned())
            }
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(res)
    }

    pub async fn list_graph(&self) -> Result<serde_json::Value, String> {
        let inner = self.inner.clone();
        let res = tokio::task::spawn_blocking(move || -> Result<serde_json::Value, String> {
            crate::database::graph::build_graph_blocking(inner)
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(res)
    }

    pub async fn find_or_generate<F, Fut>(
        &self,
        query: &Conversation,
        min_score: Option<f64>,
        mut generator: F,
        connector: Option<Arc<dyn crate::connector::ChatConnector>>,
    ) -> Result<Option<Conversation>, String>
    where
        F: FnMut(&Conversation) -> Fut,
        Fut: std::future::Future<Output = Result<Conversation, String>>,
    {
        if let Some((id, score)) = self.get_best_match(query).await? {
            if let Some(min) = min_score {
                if score.score >= min {
                    return self.get_conversation_by_id(&id).await;
                }
            } else {
                return self.get_conversation_by_id(&id).await;
            }
        }

        let generated = generator(query).await?;

        let positive = if let Some(conn) = connector {
            conn.send_and_get_feedback(&generated).await?
        } else {
            true
        };

        if positive {
            self.store_local(generated.clone()).await?;
            Ok(Some(generated))
        } else {
            Ok(Some(generated))
        }
    }

    pub async fn handle_chat_feedback<C: crate::connector::ChatConnector + 'static>(
        &self,
        conv: Conversation,
        connector: Arc<C>,
    ) -> Result<bool, String> {
        let ok = connector.send_and_get_feedback(&conv).await?;
        if ok {
            let key = format!("conv:{}", conv.id);
            let bytes = bincode::serialize(&conv).map_err(|e| e.to_string())?;
            self.storage.put_blob(&key, &bytes).await?;

            let funnel = crate::funnel::Funnel::new(&format!("funnel:{}", conv.id));
            let fbytes = bincode::serialize(&funnel).map_err(|e| e.to_string())?;
            let fkey = format!("funnel:{}", conv.id);
            self.storage.put_blob(&fkey, &fbytes).await?;
        }
        Ok(ok)
    }
}
