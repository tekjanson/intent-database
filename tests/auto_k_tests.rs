use async_trait::async_trait;
use intent_database::funnel::Embedder;
use intent_database::funnel_builder::ClusterFunnelBuilder;

struct MappingEmbedder {
    dim: usize,
}

impl MappingEmbedder {
    fn new(dim: usize) -> Self {
        Self { dim }
    }
}

#[async_trait]
impl Embedder for MappingEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let mut v = vec![0.0f32; self.dim];
        match text {
            "aa" => v[0] = 1.0,
            "bb" => v[1] = 1.0,
            "cc" => v[2] = 1.0,
            "same" => v[0] = 1.0,
            _ => v[0] = 0.1,
        }
        Ok(v)
    }
}
use intent_database::SledStorage;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_auto_k_selects_three_for_three_clusters() {
    let embed = Arc::new(MappingEmbedder::new(8));
    let mut convs = Vec::new();
    for i in 0..10 {
        convs.push(intent_database::Conversation::new(
            format!("c{}", i),
            intent_database::Intent::new("aa".to_string()),
            intent_database::Sentiment::Neutral,
        ));
    }
    for i in 10..20 {
        convs.push(intent_database::Conversation::new(
            format!("c{}", i),
            intent_database::Intent::new("bb".to_string()),
            intent_database::Sentiment::Neutral,
        ));
    }
    for i in 20..30 {
        convs.push(intent_database::Conversation::new(
            format!("c{}", i),
            intent_database::Intent::new("cc".to_string()),
            intent_database::Sentiment::Neutral,
        ));
    }
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));
    let builder = ClusterFunnelBuilder::new(0, 100);
    let funnel = builder
        .build_and_persist("ak1", &convs, embed.clone(), storage.clone())
        .await
        .expect("build");
    assert_eq!(funnel.node_ids.len(), 3);
}

#[tokio::test]
async fn test_auto_k_simple_one_cluster() {
    let embed2 = Arc::new(MappingEmbedder::new(8));
    let mut convs2 = Vec::new();
    for i in 0..15 {
        convs2.push(intent_database::Conversation::new(
            format!("d{}", i),
            intent_database::Intent::new("same".to_string()),
            intent_database::Sentiment::Neutral,
        ));
    }
    let tmp2 = TempDir::new().unwrap();
    let storage2 = Arc::new(SledStorage::new(tmp2.path().to_path_buf()));
    let builder2 = ClusterFunnelBuilder::new(0, 50);
    let funnel2 = builder2
        .build_and_persist("ak2", &convs2, embed2.clone(), storage2.clone())
        .await
        .expect("build2");
    assert_eq!(funnel2.node_ids.len(), 1);
}
