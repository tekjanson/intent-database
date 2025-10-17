use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::IntentDatabase;
use std::time::Instant;

fn main() {
    let mut db = IntentDatabase::new();

    for i in 0..10000 {
        let topic = format!("Topic {}", i);
        let t = format!("t{}", i % 10);
        let intent = Intent::new(topic).with_topics(vec![t]);
        let conv = Conversation::new(format!("id{}", i), intent, Sentiment::Neutral);
        db.store(conv).unwrap();
    }

    let qi = Intent::new("Topic 1".to_string()).with_topics(vec!["t1".to_string()]);
    let query = Conversation::new("q".to_string(), qi, Sentiment::Neutral);
    let start = Instant::now();
    let _ = db.find_similar(&query);
    println!("find_similar took: {:?}", start.elapsed());
}
