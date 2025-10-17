use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::IntentDatabase;
use std::sync::Arc;

#[tokio::test]
async fn test_db_concurrent_store_update_remove() {
    let db = Arc::new(std::sync::Mutex::new(IntentDatabase::new()));

    let mut handles = Vec::new();
    // spawn many tasks that store, update, and remove
    for i in 0..100 {
        let db = db.clone();
        handles.push(tokio::spawn(async move {
            let id = format!("c{}", i);
            let intent = Intent::new(format!("I{}", i));
            let conv = Conversation::new(id.clone(), intent, Sentiment::Neutral);
            {
                let mut g = db.lock().unwrap();
                g.store(conv).unwrap();
            }

            // update metadata
            {
                let mut g = db.lock().unwrap();
                if let Some(mut c) = g.get(&id).cloned() {
                    c.add_metadata("k".to_string(), "v".to_string());
                    g.update(c).unwrap();
                }
            }

            // remove
            {
                let mut g = db.lock().unwrap();
                g.remove(&id);
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // At the end DB should be empty
    assert_eq!(db.lock().unwrap().len(), 0);
}
