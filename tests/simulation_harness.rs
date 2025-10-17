use intent_database::ai_sim::{SimulatedConnector, SimulatedModelAdapter};
use intent_database::database::AsyncIntentDatabase;
use intent_database::storage::{SledStorage, Storage};
use intent_database::{Conversation, ConversationEntry, Intent, Sentiment};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn simulation_high_throughput_persistence() {
    // create a tempfile-backed sled path
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();

    let storage = SledStorage::new(db_path.clone());

    // start with an empty IntentDatabase in memory
    let idb = intent_database::database::IntentDatabase::new();
    let async_db = AsyncIntentDatabase::new(idb, Arc::new(storage));

    // simulated model and connector: every 3rd conversation is positive
    let model = Arc::new(SimulatedModelAdapter::new(42, "sim"));
    let connector = Arc::new(SimulatedConnector::new(model, 3));

    // generate and send 50 simulated conversations
    let mut positive_count = 0usize;
    for i in 0..50u32 {
        let intent =
            Intent::new(format!("intent-{}", i % 5)).with_topics(vec![format!("topic-{}", i % 3)]);
        let mut conv = Conversation::new(format!("conv-{}", i), intent, Sentiment::Neutral);
        conv.add_entry(ConversationEntry::new("user".to_string(), format!("hello {}", i)));

        let ok =
            async_db.handle_chat_feedback(conv.clone(), connector.clone()).await.expect("send");
        if ok {
            positive_count += 1;
            // verify storage contains the persisted conversation blob by opening
            // a fresh SledStorage for the same path (this uses the registry to
            // reuse the DB handle if already opened)
            let key = format!("conv:{}", conv.id);
            let storage2 = intent_database::storage::SledStorage::new(db_path.clone());
            let got = storage2.get_blob(&key).await.expect("get blob");
            assert!(got.is_some());
        }
    }

    // ensure we saw some positives according to the pattern (every 3rd of 50 -> ~17)
    assert!(positive_count > 0);
}
