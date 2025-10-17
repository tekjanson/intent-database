use async_trait::async_trait;
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::funnel::HashEmbedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;
use intent_database::storage::{SledStorage, Storage};
use std::sync::Arc;
use tempfile::tempdir;

// Reuse embedder implementations from the library where available
struct IdenticalEmbedder {
    v: Vec<f32>,
}
#[async_trait]
impl intent_database::funnel::Embedder for IdenticalEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(self.v.clone())
    }
}
struct ZeroEmbedder {
    dim: usize,
}
#[async_trait]
impl intent_database::funnel::Embedder for ZeroEmbedder {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, String> {
        Ok(vec![0.0f32; self.dim])
    }
}

// failing storage
struct FailingStorage {}
#[async_trait]
impl Storage for FailingStorage {
    async fn put_blob(&self, _key: &str, _data: &[u8]) -> Result<(), String> {
        Err("fail".to_string())
    }

    async fn get_blob(&self, _key: &str) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }
}

#[tokio::test]
async fn contract_permutations() {
    let ks = [1usize, 2usize, 3usize];

    let datasets: Vec<Vec<Conversation>> = vec![
        vec![], // empty
        vec![
            // small distinct
            Conversation::new(
                "a".to_string(),
                Intent::new("billing".to_string()),
                Sentiment::Neutral,
            ),
            Conversation::new(
                "b".to_string(),
                Intent::new("login".to_string()),
                Sentiment::Neutral,
            ),
        ],
        vec![
            // identical intents
            Conversation::new(
                "i1".to_string(),
                Intent::new("same".to_string()),
                Sentiment::Neutral,
            ),
            Conversation::new(
                "i2".to_string(),
                Intent::new("same".to_string()),
                Sentiment::Neutral,
            ),
            Conversation::new(
                "i3".to_string(),
                Intent::new("same".to_string()),
                Sentiment::Neutral,
            ),
        ],
    ];

    for ei in 0..3usize {
        for &k in ks.iter() {
            for (di, ds) in datasets.iter().enumerate() {
                // construct embedder per permutation
                let embedder: Arc<dyn intent_database::funnel::Embedder> = match ei {
                    0 => Arc::new(HashEmbedder::new(8)),
                    1 => Arc::new(IdenticalEmbedder { v: vec![1.0; 8] }),
                    2 => Arc::new(ZeroEmbedder { dim: 8 }),
                    _ => unreachable!(),
                };

                // test with working storage
                let dir = tempdir().unwrap();
                let path = dir.path().join(format!("contract_{}_{}_{}", ei, k, di));
                let storage = Arc::new(SledStorage::new(path));
                let builder = ClusterFunnelBuilder::new(k, 5);
                let eid = format!("c_{}_{}_{}", ei, k, di);
                let res =
                    builder.build_and_persist(&eid, ds, embedder.clone(), storage.clone()).await;
                // expectations
                if ds.is_empty() {
                    assert!(res.is_ok());
                    let funnel = res.unwrap();
                    assert!(funnel.node_ids.is_empty());
                } else {
                    assert!(res.is_ok());
                    let funnel = res.unwrap();
                    assert!(!funnel.node_ids.is_empty());
                }

                // test with failing storage - expect Err
                let storage2 = Arc::new(FailingStorage {});
                let res2 =
                    builder.build_and_persist("failtest", ds, embedder.clone(), storage2).await;
                assert!(res2.is_err());
            }
        }
    }
}

// end of contract tests
