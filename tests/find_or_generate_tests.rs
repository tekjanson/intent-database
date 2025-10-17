use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::AsyncIntentDatabase;
use intent_database::storage::SledStorage;
// removed unused import: intent_database::funnel::ModelAdapter
use async_trait::async_trait;
use intent_database::connector::ChatConnector;
use std::sync::Arc;
use tempfile::TempDir;

struct Generator;

impl Generator {
    async fn gen_conv(&self, query: &Conversation) -> Result<Conversation, String> {
        let mut out = Conversation::new(
            format!("gen-{}", query.id),
            query.intent.clone(),
            query.sentiment.clone(),
        );
        out.add_entry(intent_database::conversation::ConversationEntry::new(
            "assistant".to_string(),
            "generated".to_string(),
        ));
        Ok(out)
    }
}

struct PosConnector;

#[async_trait]
impl ChatConnector for PosConnector {
    async fn send_and_get_feedback(&self, _conv: &Conversation) -> Result<bool, String> {
        Ok(true)
    }
}

struct NegConnector;

#[async_trait]
impl ChatConnector for NegConnector {
    async fn send_and_get_feedback(&self, _conv: &Conversation) -> Result<bool, String> {
        Ok(false)
    }
}

#[tokio::test]
async fn find_or_generate_short_circuits_to_stored() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(db, Arc::new(storage));
    let async_db = Arc::new(async_db);

    let intent = Intent::new("Short circuit".to_string());
    let stored = Conversation::new("s1".to_string(), intent.clone(), Sentiment::Positive);
    async_db.store_local(stored.clone()).await.unwrap();

    let query = Conversation::new("q1".to_string(), intent.clone(), Sentiment::Positive);
    let res = async_db
        .find_or_generate(
            &query,
            None,
            |_q| async { Err("should not be called".to_string()) },
            None,
        )
        .await
        .unwrap();
    assert!(res.is_some());
    let got = res.unwrap();
    assert_eq!(got.id, "s1");
}

#[tokio::test]
async fn find_or_generate_generates_and_persists_on_positive() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(db, Arc::new(storage));
    let async_db = Arc::new(async_db);

    let intent = Intent::new("Generate me".to_string());
    let query = Conversation::new("qgen".to_string(), intent.clone(), Sentiment::Neutral);

    let gen = std::sync::Arc::new(Generator {});
    let res = async_db
        .find_or_generate(
            &query,
            None,
            move |q: &Conversation| {
                let q_owned = q.clone();
                let gen = gen.clone();
                async move { gen.gen_conv(&q_owned).await }
            },
            Some(Arc::new(PosConnector)),
        )
        .await
        .unwrap();
    assert!(res.is_some());
    // ensure persisted
    let persisted = async_db.get_conversation_by_id("gen-qgen").await.unwrap();
    assert!(persisted.is_some());
}

#[tokio::test]
async fn find_or_generate_generates_but_does_not_persist_on_negative() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(db, Arc::new(storage));
    let async_db = Arc::new(async_db);

    let intent = Intent::new("Generate me".to_string());
    let query = Conversation::new("qgen2".to_string(), intent.clone(), Sentiment::Neutral);

    let gen = std::sync::Arc::new(Generator {});
    let res = async_db
        .find_or_generate(
            &query,
            None,
            move |q: &Conversation| {
                let q_owned = q.clone();
                let gen = gen.clone();
                async move { gen.gen_conv(&q_owned).await }
            },
            Some(Arc::new(NegConnector)),
        )
        .await
        .unwrap();
    assert!(res.is_some());
    // ensure not persisted
    let persisted = async_db.get_conversation_by_id("gen-qgen2").await.unwrap();
    assert!(persisted.is_none());
}
