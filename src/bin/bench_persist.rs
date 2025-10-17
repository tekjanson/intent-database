use intent_database::database::IntentDatabase;
use std::time::Instant;

fn main() {
    let db = IntentDatabase::new();
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = bincode::serialize(&db).unwrap();
    }
    println!("persist loop took: {:?}", start.elapsed());
}
