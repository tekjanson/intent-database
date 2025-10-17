use async_trait::async_trait;
use intent_database::connector::ChatConnector;
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
impl ChatConnector for CountingConnector {
    async fn send_and_get_feedback(&self, conv: &Conversation) -> Result<bool, String> {
        // Use the model to simulate work (increments count)
        let _ = self.model.generate_question(&conv.intent.purpose).await?;
        Ok(true)
    }
}

#[tokio::test]
async fn short_circuit_prevents_model_call() {
    // prepare storage in a tempdir
    let tmp = TempDir::new().expect("tempdir");
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = Arc::new(AsyncIntentDatabase::new(db, Arc::new(storage)));

    // Insert a conversation that will match the query (same purpose)
    let intent = Intent::new("Order pizza".to_string());
    let conv_stored = Conversation::new("c1".to_string(), intent.clone(), Sentiment::Positive);
    // store via helper
    async_db.store_local(conv_stored.clone()).await.expect("store");

    // Prepare counting adapter and connector
    let counter = Arc::new(AtomicUsize::new(0));
    let model = Arc::new(CountingAdapter::new(counter.clone()));
    let inner_connector = Arc::new(CountingConnector { model });

    let sc = ShortCircuitingConnector::new(async_db.clone(), inner_connector.clone());

    // Create a query conversation with the same purpose so matcher will find it
    let query = Conversation::new("q1".to_string(), intent, Sentiment::Positive);

    let res = sc.send_and_get_feedback(&query).await.expect("connector call");
    assert!(res, "short-circuit should report positive feedback");

    // Ensure inner model was NOT called
    assert_eq!(counter.load(Ordering::SeqCst), 0, "model should not have been invoked");
}
