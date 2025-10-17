use criterion::{criterion_group, criterion_main, Criterion};
use intent_database::database::IntentDatabase;
use tempfile::tempdir;

pub fn bench_persist(c: &mut Criterion) {
    let db = IntentDatabase::new();
    let _dir = tempdir().unwrap();

    c.bench_function("persist_bincode", |b| {
        b.iter(|| {
            let _ = bincode::serialize(&db).unwrap();
        })
    });
}

criterion_group!(benches, bench_persist);
criterion_main!(benches);
