use criterion::{Criterion, criterion_group, criterion_main};
use std::{sync::Arc, thread};
use txmap::{locks::mutex_policy::MutexPolicy, prelude::*};

fn shards(c: &mut Criterion) {
    for sc in vec![
        ShardCount::_8,
        ShardCount::_16,
        ShardCount::_32,
        ShardCount::_64,
        ShardCount::_128,
    ] {
        let txmap = TxMap::with_lock_policy::<MutexPolicy>(sc);
        c.bench_function(&format!("txmap_insert_shards_{}", sc), |b| {
            b.iter(|| {
                let key = std::hint::black_box("key".to_string());
                txmap.insert(key, 42);
            });
        });
    }
}

fn concurrent_shards(c: &mut Criterion) {
    let num_threads = 8;
    let ops_per_thread = 1_000;

    for sc in vec![
        ShardCount::_8,
        ShardCount::_16,
        ShardCount::_32,
        ShardCount::_64,
        ShardCount::_128,
    ] {
        let map = Arc::new(TxMap::new(sc));
        c.bench_function(&format!("txmap_concurrent_insert_shards_{}", sc), |b| {
            b.iter(|| {
                let handles: Vec<_> = (0..num_threads)
                    .map(|_| {
                        let map = map.clone();
                        thread::spawn(move || {
                            for i in 0..ops_per_thread {
                                let key = std::hint::black_box(format!(
                                    "key_{:?}_{}",
                                    thread::current().id(),
                                    i
                                ));
                                map.insert(key, 42);
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
}

criterion_group!(benches, shards, concurrent_shards);
criterion_main!(benches);
