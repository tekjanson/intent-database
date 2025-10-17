use criterion::{criterion_group, criterion_main, Criterion};
use intent_database::conversation::{Conversation, Intent, Sentiment};
use intent_database::database::IntentDatabase;

pub fn bench_find_similar(c: &mut Criterion) {
    let mut db = IntentDatabase::new();

    for i in 0..1000 {
        let topic = format!("Topic {}", i);
        let t = format!("t{}", i % 10);
        let intent = Intent::new(topic).with_topics(vec![t]);
        let conv = Conversation::new(format!("id{}", i), intent, Sentiment::Neutral);
        db.store(conv).unwrap();
    }

    let query_intent = Intent::new("Topic 1".to_string()).with_topics(vec!["t1".to_string()]);
    let query = Conversation::new("q".to_string(), query_intent, Sentiment::Neutral);

    c.bench_function("find_similar_linear", |b| {
        b.iter(|| {
            let _ = db.find_similar(&query);
        })
    });
}

criterion_group!(benches, bench_find_similar);
criterion_main!(benches);
