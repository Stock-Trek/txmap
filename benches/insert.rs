use criterion::{Criterion, criterion_group, criterion_main};
use hashbrown::HashMap;
use std::{sync::Arc, thread};
use txmap::prelude::*;

fn insert(c: &mut Criterion) {
    let mut hashbrownmap = HashMap::new();
    let map: TxMap<&str, u64> = TxMap::new();
    let tx = map
        .prepared_tx(&Insert::SCHEMA)
        .insert_with(Insert::a, |_, _, _| 1)
        .into_transaction();
    let tx5 = map
        .prepared_tx(&Insert::SCHEMA)
        .insert_with(Insert::a, |_, _, _| 1)
        .insert_with(Insert::a, |_, _, _| 1)
        .insert_with(Insert::a, |_, _, _| 1)
        .insert_with(Insert::a, |_, _, _| 1)
        .insert_with(Insert::a, |_, _, _| 1)
        .into_transaction();

    c.bench_function("hashbrownmap_insert", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            hashbrownmap.insert(key, 42);
        });
    });
    c.bench_function("txmap_insert", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            map.insert(key, 42);
        });
    });
    c.bench_function("txmap_insert_tx", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            let _ = tx.execute(InsertKeys { a: key }, InsertParams {});
        });
    });
    c.bench_function("txmap_insert_tx5", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            let _ = tx5.execute(InsertKeys { a: key }, InsertParams {});
        });
    });
    c.bench_function("txmap_insert_immediate", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            let _ = map
                .immediate_tx::<()>()
                .insert_with(key, |_, _| 1)
                .execute();
        });
    });
    c.bench_function("txmap_insert_immediate5", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key");
            let _ = map
                .immediate_tx::<()>()
                .insert_with(key, |_, _| 1)
                .insert_with(key, |_, _| 1)
                .insert_with(key, |_, _| 1)
                .insert_with(key, |_, _| 1)
                .insert_with(key, |_, _| 1)
                .execute();
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
                            let _ = tx.execute(InsertKeys { a: key }, InsertParams {});
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
