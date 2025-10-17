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

struct CountingAdapter2 {
    counter: Arc<AtomicUsize>,
}

impl CountingAdapter2 {
    fn new(counter: Arc<AtomicUsize>) -> Self {
        Self { counter }
    }
}

#[async_trait]
impl ModelAdapter for CountingAdapter2 {
    async fn generate_question(&self, _context: &str) -> Result<String, String> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok("counted2".to_string())
    }
}

struct CountingConnector2 {
    model: Arc<dyn ModelAdapter>,
}

#[async_trait]
impl intent_database::connector::ChatConnector for CountingConnector2 {
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String> {
        let _ = self.model.generate_question(&conv.intent.purpose).await?;
        Ok(false)
    }
}

#[tokio::test]
async fn no_match_delegates_to_inner_connector() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = Arc::new(AsyncIntentDatabase::new(db, Arc::new(storage)));

    // no stored conversations inserted

    let counter = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(CountingAdapter2::new(counter.clone()));
    let inner = Arc::new(CountingConnector2 { model });

    let sc = ShortCircuitingConnector::new(async_db.clone(), inner);
    let query = Conversation::new(
        "q-nomatch".to_string(),
        Intent::new("UniqueIntent".to_string()),
        Sentiment::Neutral,
    );

    let res = sc.send_and_get_feedback_and_maybe_short_circuit(&query).await.unwrap();
    assert!(res.matched.is_none(), "expected no matched stored conversation");
    assert!(!res.feedback, "expected inner connector to return false");
    assert_eq!(counter.load(Ordering::SeqCst), 1, "model should have been invoked once");
}
