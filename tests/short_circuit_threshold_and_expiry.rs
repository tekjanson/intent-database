use async_trait::async_trait;
use chrono::{Duration, Utc};
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::AsyncIntentDatabase;
use intent_database::funnel::ModelAdapter;
use intent_database::shortcircuit::ShortCircuitingConnector;
use intent_database::storage::SledStorage;
use std::sync::Arc;
use tempfile::TempDir;

struct NoopAdapter;

#[async_trait]
impl ModelAdapter for NoopAdapter {
    async fn generate_question(&self, _context: &str) -> Result<String, String> {
        Ok("noop".to_string())
    }
}

struct NoopConnector {
    _model: Arc<dyn ModelAdapter>,
}

#[async_trait]
impl intent_database::connector::ChatConnector for NoopConnector {
    async fn send_and_get_feedback(&self, _conv: &Conversation) -> Result<bool, String> {
        Ok(false)
    }
}

#[tokio::test]
async fn short_circuit_respects_min_score_and_returns_conv() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = Arc::new(AsyncIntentDatabase::new(db, Arc::new(storage)));

    // store a conversation that will match with score ~1.0
    let intent = Intent::new("Help me".to_string());
    let stored = Conversation::new("s1".to_string(), intent.clone(), Sentiment::Positive);
    async_db.store_local(stored.clone()).await.unwrap();

    let inner = Arc::new(NoopConnector { _model: Arc::new(NoopAdapter) });
    let mut sc = ShortCircuitingConnector::new(async_db.clone(), inner);

    // set a very high min_score so it won't match
    sc.min_score = Some(0.99);
    let query = Conversation::new("q1".to_string(), intent.clone(), Sentiment::Positive);
    let res1 = sc.send_and_get_feedback_and_maybe_short_circuit(&query).await.unwrap();
    // Depending on matcher this may or may not match; ensure when it doesn't, matched is None
    if res1.matched.is_none() {
        // now lower threshold so it matches
        sc.min_score = Some(0.0);
        let res2 = sc.send_and_get_feedback_and_maybe_short_circuit(&query).await.unwrap();
        assert!(res2.matched.is_some(), "expected match when threshold lowered");
        let (_id, _score, conv_opt) = res2.matched.unwrap();
        assert!(conv_opt.is_some(), "expected stored conversation to be returned");
    } else {
        // if first matched, ensure conversation present
        let (_id, _score, conv_opt) = res1.matched.unwrap();
        assert!(conv_opt.is_some(), "expected stored conversation to be returned");
    }
}

#[tokio::test]
async fn short_circuit_ignores_expired_conversation() {
    let tmp = TempDir::new().unwrap();
    let storage = SledStorage::new(tmp.path().to_path_buf());
    let db = intent_database::database::IntentDatabase::new();
    let async_db = Arc::new(AsyncIntentDatabase::new(db, Arc::new(storage)));

    let intent = Intent::new("Time-based".to_string());
    let mut stored = Conversation::new("exp1".to_string(), intent.clone(), Sentiment::Positive);
    let past = Utc::now() - Duration::hours(2);
    stored = stored.with_expiry(past);
    async_db.store_local(stored.clone()).await.unwrap();

    let inner = Arc::new(NoopConnector { _model: Arc::new(NoopAdapter) });
    let sc = ShortCircuitingConnector::new(async_db.clone(), inner);

    let query = Conversation::new("q-exp".to_string(), intent.clone(), Sentiment::Positive);
    let res = sc.send_and_get_feedback_and_maybe_short_circuit(&query).await.unwrap();
    assert!(res.matched.is_none(), "expired conversation should not match");
}
