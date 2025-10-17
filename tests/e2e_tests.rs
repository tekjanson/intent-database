use intent_database::database::AsyncIntentDatabase;
use intent_database::funnel::StubEmbedder;
use intent_database::storage::Storage;
use intent_database::Conversation;
use intent_database::{MockConnector, SledStorage, TemplateModelAdapter};
use std::sync::Arc;
use tempfile::TempDir;

fn make_conversation(id: &str, purpose: &str) -> Conversation {
    let intent = intent_database::Intent::new(purpose.to_string());
    Conversation::new(id.to_string(), intent, intent_database::Sentiment::Neutral)
}

#[tokio::test]
async fn test_e2e_chat_persist_and_funnel_activation() {
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(SledStorage::new(tmp.path().to_path_buf()));

    let db = intent_database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(db, storage.clone());

    // conversation and positive feedback
    let conv = make_conversation("x1", "billing issue");
    let connector = Arc::new(MockConnector { positive: true });

    let res = async_db.handle_chat_feedback(conv.clone(), connector).await.expect("handle");
    assert!(res, "expected positive feedback to return true");

    // check blobs exist
    let key = format!("conv:{}", conv.id);
    let blob = storage.get_blob(&key).await.expect("get blob");
    assert!(blob.is_some());

    // funnel persisted
    let fkey = format!("funnel:{}", conv.id);
    let fblob = storage.get_blob(&fkey).await.expect("get funnel");
    assert!(fblob.is_some());

    // Activate funnel session using TemplateModelAdapter and StubEmbedder
    let funnel = intent_database::funnel::Funnel::new(&format!("funnel:{}", conv.id));
    let model = Arc::new(TemplateModelAdapter::new());
    let embedder = Arc::new(StubEmbedder {});
    let mut session = std::sync::Arc::new(funnel).activate(embedder, model).await;
    let q = session.next_question();
    assert!(q.is_some());
}
