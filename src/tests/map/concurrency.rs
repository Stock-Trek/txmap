use crate::{
    prelude::*,
    tests::{
        creators::*,
        types::{
            Increment, IncrementKeys, IncrementParams, Transfer, TransferKeys, TransferParams,
        },
    },
};
use std::{
    sync::{Arc, Barrier},
    thread,
};

const THREAD_COUNT: u64 = 8;
const LONG_LOOP: u64 = 10_000;
const RANDOM_NAME_COUNT: usize = 2;

#[test]
fn concurrent_inserts_are_thread_safe() {
    let map = Arc::new(empty_typed_map::<u64, u64>());
    let mut handles = Vec::new();
    for t in 0..THREAD_COUNT {
        let m = map.clone();
        handles.push(thread::spawn(move || {
            for i in 0..LONG_LOOP {
                m.insert(i * 8 + t, t);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(map.len() as u64, THREAD_COUNT * LONG_LOOP);
}

#[test]
fn concurrent_transactions_dont_deadlock() {
    let map = Arc::new(map_alice_bob_chuck_dave(
        1_000_000, 1_000_000, 1_000_000, 1_000_000,
    ));
    let barrier = Arc::new(Barrier::new(THREAD_COUNT as usize));
    let mut handles = Vec::new();
    for _ in 0..THREAD_COUNT {
        let m = map.clone();
        let b = barrier.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..LONG_LOOP {
                let [from, to] = random_names::<RANDOM_NAME_COUNT>();
                let _ = m
                    .prepared_tx(&Transfer::SCHEMA)
                    .modify(Transfer::from, |_k, v, p, _s| *v -= p.amount)
                    .modify(Transfer::to, |_k, v, p, _s| *v += p.amount)
                    .into_transaction()
                    .execute(TransferKeys { from, to }, TransferParams { amount: 1 });
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let total = map.fold(0u64, |_, v| Some(*v), |total, v| total + v);
    assert_eq!(total, 4_000_000);
}

#[test]
fn concurrent_reads_and_writes() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let map = Arc::new(empty_typed_map::<u64, u64>());
    let done = Arc::new(AtomicBool::new(false));

    let mw = map.clone();
    let dw = done.clone();
    let writer = thread::spawn(move || {
        for i in 0..LONG_LOOP {
            mw.insert(i, i * 2);
        }
        dw.store(true, Ordering::SeqCst);
    });

    let mr = map.clone();
    let dr = done.clone();
    let reader = thread::spawn(move || {
        while !dr.load(Ordering::SeqCst) {
            let _ = mr.fold(0u64, |_k, v| Some(*v), |acc, v| acc + v);
        }
    });

    writer.join().unwrap();
    reader.join().unwrap();

    let total = map.fold(0u64, |_, v| Some(*v), |total, v| total + v);
    assert_eq!(total, (LONG_LOOP - 1) * LONG_LOOP);
}

#[test]
fn atomic_transaction_isolation() {
    let map = Arc::new(empty_typed_map::<u64, u64>());
    map.insert(1, 0);
    let map_clone1 = map.clone();
    let map_clone2 = map.clone();
    let barrier = Arc::new(Barrier::new(2));
    let b1 = barrier.clone();
    let b2 = barrier.clone();

    let h1 = thread::spawn(move || {
        b1.wait();
        for _ in 0..LONG_LOOP {
            let _ = map_clone1
                .prepared_tx(&Increment::SCHEMA)
                .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                .into_transaction()
                .execute(IncrementKeys { k: 1 }, IncrementParams {});
        }
    });

    let h2 = thread::spawn(move || {
        b2.wait();
        for _ in 0..LONG_LOOP {
            let _ = map_clone2
                .prepared_tx(&Increment::SCHEMA)
                .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                .into_transaction()
                .execute(IncrementKeys { k: 1 }, IncrementParams {});
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    assert_eq!(map.get_with(&1, |v| *v), Some(LONG_LOOP * 2));
}

/// Small xorshift PRNG so the Zipfian stress test has no extra dependencies.
fn next_rand(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Picks a key with an 80/20 (roughly Zipfian) distribution: most operations
/// hit a small hot set while the rest spread over the whole key space.
fn zipfian_key(state: &mut u64, key_count: u64, hot_count: u64) -> u64 {
    if next_rand(state) % 100 < 80 {
        next_rand(state) % hot_count
    } else {
        next_rand(state) % key_count
    }
}

#[test]
fn zipfian_transactions_with_topology_churn() {
    use std::sync::atomic::{AtomicBool, Ordering};

    const KEYS: u64 = 128;
    const HOT: u64 = 16;
    const WORKERS: u64 = 4;
    const OPS: u64 = 2_000;

    let map = Arc::new(
        TxMapBuilder::default()
            .with_shards(Shards::_128)
            .build::<u64, u64>(),
    );
    for key in 0..KEYS {
        map.insert(key, 0);
    }

    let done = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();

    // Workers drive many small Zipfian transactions, which retry across
    // splits and merges thanks to the version re-check.
    for worker in 0..WORKERS {
        let m = map.clone();
        handles.push(thread::spawn(move || {
            let mut state = 0x9E37_79B9_7F4A_7C15 ^ (worker + 1);
            for _ in 0..OPS {
                let key = zipfian_key(&mut state, KEYS, HOT);
                let _ = m
                    .prepared_tx(&Increment::SCHEMA)
                    .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                    .into_transaction()
                    .execute(IncrementKeys { k: key }, IncrementParams {});
            }
        }));
    }

    // Concurrently churn topology: merge a branch, then split it back.
    let churn_map = map.clone();
    let churn_done = done.clone();
    let churn = thread::spawn(move || {
        let mut state = 0x1234_5678_9ABC_DEF0;
        while !churn_done.load(Ordering::Relaxed) {
            let key = zipfian_key(&mut state, KEYS, HOT);
            let _ = churn_map.custodian.merge_leaves(&churn_map.indexer, None);
            let hash = churn_map.indexer.hash(&key);
            let leaf = churn_map.custodian.route(hash).0;
            let _ = churn_map.custodian.split_leaf(&churn_map.indexer, leaf);
            thread::yield_now();
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    done.store(true, Ordering::Relaxed);
    churn.join().unwrap();

    let total = map.fold(0u64, |_k, v| Some(*v), |acc, v| acc + v);
    assert_eq!(total, WORKERS * OPS);
}
