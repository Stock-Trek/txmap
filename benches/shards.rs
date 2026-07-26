use criterion::{Criterion, criterion_group, criterion_main};
use std::{sync::Arc, thread};
use txmap::{lock_policies::mutex_policy::MutexPolicy, prelude::*};

fn shards(c: &mut Criterion) {
    for shards in [
        Shards::_8,
        Shards::_16,
        Shards::_32,
        Shards::_64,
        Shards::_128,
    ] {
        let txmap = TxMap::with_lock_policy::<MutexPolicy>(shards);
        c.bench_function(&format!("txmap_insert_shards_{}", shards), |b| {
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

    for shards in [
        Shards::_8,
        Shards::_16,
        Shards::_32,
        Shards::_64,
        Shards::_128,
    ] {
        let map = Arc::new(TxMap::new(shards));
        c.bench_function(&format!("txmap_concurrent_insert_shards_{}", shards), |b| {
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
