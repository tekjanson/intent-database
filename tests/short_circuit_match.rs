use async_trait::async_trait;
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::AsyncIntentDatabase;
use intent_database::funnel::ModelAdapter;
use intent_database::shortcircuit::ShortCircuitingConnector;
use intent_database::storage::SledStorage;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tempfile::TempDir;

struct CountingAdapter {
    counter: Arc<AtomicUsize>,
}

impl CountingAdapter {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        Self { counter }
    }
}

#[async_trait]
impl ModelAdapter for CountingAdapter {
    async fn generate_question(&self, _context: &str) -> Result<String, String> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok("counted".to_string())
    }
}

struct CountingConnector {
    model: Arc<dyn ModelAdapter>,
}

#[async_trait]
impl intent_database::connector::ChatConnector for CountingConnector {
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String> {
        let _ = self.model.generate_question(&conv.intent.purpose).await?;
        Ok(true)
    }
}

#[tokio::test]
async fn short_circuit_returns_matched_and_prevents_model() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = Arc::new(AsyncIntentDatabase::new(db, Arc::new(storage)));

    let intent = Intent::new("Billing issue".to_string());
    let stored = Conversation::new("stored-1".to_string(), intent.clone(), Sentiment::Positive);
    async_db.store_local(stored.clone()).await.unwrap();

    let counter = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(CountingAdapter::new(counter.clone()));
    let inner = Arc::new(CountingConnector { model });

    let sc = ShortCircuitingConnector::new(async_db.clone(), inner);
    let query = Conversation::new("q2".to_string(), intent, Sentiment::Positive);

    let res = sc.send_and_get_feedback_and_maybe_short_circuit(&query).await.unwrap();
    assert!(res.matched.is_some(), "expected a matched stored conversation");
    assert!(res.feedback, "feedback should be true for matched conversation");
    assert_eq!(counter.load(Ordering::SeqCst), 0, "model should not have been invoked");
}
