use chrono::{Duration, Utc};
use intent_database::database::IntentDatabase;
use intent_database::{Conversation, Intent, Sentiment};

#[test]
fn e2e_purge_expired_conversation() {
    let mut db = IntentDatabase::new();
    let expired_intent = Intent::new("Old fact".to_string());
    let expired_conv =
        Conversation::new("conv-expired".to_string(), expired_intent, Sentiment::Neutral)
            .with_expiry(Utc::now() - Duration::hours(1));

    db.store(expired_conv).unwrap();
    let cleaned = db.cleanup_expired();
    assert_eq!(cleaned, 1);
    assert!(db.is_empty());
}
