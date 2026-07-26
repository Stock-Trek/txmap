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
    // Start with sufficient funds to avoid underflow when subtracting
    let map = Arc::new(map_alice_bob_chuck_dave(1_000_000, 1_000_000, 1_000_000, 1_000_000));
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
                    .prepare_transaction(&Transfer::SCHEMA)
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
    // Funds are conserved: each transfer subtracts 1 from `from` and adds 1 to `to`
    // So total sum across all keys should remain equal to the initial total
    assert_eq!(total, 4_000_000);
}

#[test]
fn concurrent_reads_and_writes() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let map = Arc::new(empty_typed_map::<u64, u64>());
    let done = Arc::new(AtomicBool::new(false));

    // Writer thread
    let mw = map.clone();
    let dw = done.clone();
    let writer = thread::spawn(move || {
        for i in 0..LONG_LOOP {
            mw.insert(i, i * 2);
        }
        dw.store(true, Ordering::SeqCst);
    });

    // Reader thread
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
    // Sum of (i * 2) for i in 0..LONG_LOOP
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
                .prepare_transaction(&Increment::SCHEMA)
                .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                .into_transaction()
                .execute(IncrementKeys { k: 1 }, IncrementParams {});
        }
    });

    let h2 = thread::spawn(move || {
        b2.wait();
        for _ in 0..LONG_LOOP {
            let _ = map_clone2
                .prepare_transaction(&Increment::SCHEMA)
                .modify(Increment::k, |_k, v, _p, _s| *v += 1)
                .into_transaction()
                .execute(IncrementKeys { k: 1 }, IncrementParams {});
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    assert_eq!(map.get_with(&1, |v| *v), Some(LONG_LOOP * 2));
}
