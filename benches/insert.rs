use criterion::{Criterion, criterion_group, criterion_main};
use hashbrown::HashMap;
use std::{hint::black_box, sync::Arc, thread, time::Duration};
use txmap::prelude::*;

fn insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    group.bench_function("hashbrownmap_insert", |b| {
        let mut hashbrownmap = HashMap::new();
        b.iter(|| {
            let key = black_box("key");
            black_box(hashbrownmap.insert(key, 42));
        });
    });
    group.bench_function("txmap_insert", |b| {
        let map: TxMap<&str, u64> = TxMap::new();
        b.iter(|| {
            let key = black_box("key");
            black_box(map.insert(key, 42));
        });
    });
    group.bench_function("txmap_insert_prepared", |b| {
        let map: TxMap<&str, u64> = TxMap::new();
        let tx = map
            .prepared_tx(&Insert::SCHEMA)
            .insert_with(Insert::a, |_, _, _| 1)
            .into_transaction();
        b.iter(|| {
            let key = black_box("key");
            black_box(tx.execute(InsertKeys { a: key }, InsertParams {}));
        });
    });
    group.bench_function("txmap_insert_prepared10", |b| {
        let map: TxMap<&str, u64> = TxMap::new();
        let tx = map
            .prepared_tx(&Insert::SCHEMA)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .insert_with(Insert::a, |_, _, _| 1)
            .into_transaction();
        b.iter(|| {
            let key = black_box("key");
            black_box(tx.execute(InsertKeys { a: key }, InsertParams {}));
        });
    });
    group.bench_function("txmap_insert_immediate", |b| {
        let map: TxMap<&str, u64> = TxMap::new();
        b.iter(|| {
            let key = black_box("key");
            black_box(
                map.immediate_tx::<()>()
                    .insert_with(key, |_, _| 1)
                    .execute(),
            );
        });
    });
    group.bench_function("txmap_insert_immediate10", |b| {
        let map: TxMap<&str, u64> = TxMap::new();
        b.iter(|| {
            let key = black_box("key");
            black_box(
                map.immediate_tx::<()>()
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .insert_with(key, |_, _| 1)
                    .execute(),
            );
        });
    });
}

tx_schema! {
    Insert,
    keys: [a],
    params: {},
    state: {},
}

fn concurrent_insert(c: &mut Criterion) {
    let num_threads = 8;
    let ops_per_thread = 10_000;
    let map: Arc<TxMap<String, u64>> = Arc::new(TxMap::new());

    c.bench_function("txmap_concurrent_insert", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..num_threads)
                .map(|_| {
                    let map = map.clone();
                    thread::spawn(move || {
                        let tx = map
                            .prepared_tx(&Insert::SCHEMA)
                            .insert_with(Insert::a, |_, _, _| 1)
                            .into_transaction();
                        for i in 0..ops_per_thread {
                            let key = std::hint::black_box(format!(
                                "key_{:?}_{}",
                                thread::current().id(),
                                i
                            ));
                            black_box(tx.execute(InsertKeys { a: key }, InsertParams {}));
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        })
    });
}

criterion_group!(benches, insert, concurrent_insert);
criterion_main!(benches);
