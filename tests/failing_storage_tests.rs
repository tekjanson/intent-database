use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::Conversation;

use async_trait::async_trait;
use intent_database::storage::Storage;
use std::sync::Arc;

fn make_conversation(id: &str, purpose: &str) -> Conversation {
    let intent = intent_database::Intent::new(purpose.to_string());
    Conversation::new(id.to_string(), intent, intent_database::Sentiment::Neutral)
}

struct FailStorage;

#[async_trait]
impl Storage for FailStorage {
    async fn put_blob(&self, _key: &str, _data: &[u8]) -> Result<(), String> {
        Err("simulated storage failure".to_string())
    }
    async fn get_blob(&self, _key: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
}

#[tokio::test]
async fn test_build_propagates_storage_failure() {
    let storage = Arc::new(FailStorage {});
    let embed = Arc::new(HashEmbedder::new(8));
    let convs = vec![make_conversation("c1", "a")];
    let builder = ClusterFunnelBuilder::new(2, 10);
    let res = builder.build_and_persist("fail", &convs, embed.clone(), storage.clone()).await;
    assert!(res.is_err());
}
