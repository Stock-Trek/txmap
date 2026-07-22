use criterion::{Criterion, criterion_group, criterion_main};
use dashmap::DashMap;
use hashbrown::HashMap;
use std::{sync::Arc, thread};
use txmap::prelude::*;

fn insert(c: &mut Criterion) {
    let dashmap = DashMap::<String, i32>::new();
    let mut hashbrownmap = HashMap::<String, i32>::new();
    let txmap = TxMap::new(ShardCount::_8);

    c.bench_function("dashmap_insert", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key".to_string());
            dashmap.insert(key, 42);
        });
    });
    c.bench_function("hashbrownmap_insert", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key".to_string());
            hashbrownmap.insert(key, 42);
        });
    });
    c.bench_function("txmap_insert", |b| {
        b.iter(|| {
            let key = std::hint::black_box("key".to_string());
            txmap.insert(key, 42);
        });
    });
}

fn concurrent_insert(c: &mut Criterion) {
    let num_threads = 8;
    let ops_per_thread = 1_000;
    let dashmap = Arc::new(DashMap::<String, i32>::new());
    let map = Arc::new(TxMap::new(ShardCount::_8));

    c.bench_function("dashmap_concurrent_insert", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..num_threads)
                .map(|_| {
                    let map = dashmap.clone();
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
    c.bench_function("txmap_concurrent_insert", |b| {
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

criterion_group!(benches, insert, concurrent_insert);
criterion_main!(benches);
