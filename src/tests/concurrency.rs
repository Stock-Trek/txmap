#[cfg(test)]
mod tests {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder},
        tests::creators::creators::{empty_typed_map, map_alice_bob_chuck_dave, random_names},
    };
    use std::{
        sync::{Arc, Barrier},
        thread,
    };

    const THREAD_COUNT: u64 = 8;
    const LONG_LOOP: u64 = 10_000;
    const RANDOM_NAME_COUNT: usize = 3;

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
        let map = Arc::new(map_alice_bob_chuck_dave(0, 0, 0, 0));
        let barrier = Arc::new(Barrier::new(THREAD_COUNT as usize));
        let mut handles = Vec::new();
        for _ in 0..THREAD_COUNT {
            let m = map.clone();
            let b = barrier.clone();
            handles.push(thread::spawn(move || {
                b.wait();
                for _ in 0..LONG_LOOP {
                    let [a, b, c] = random_names::<RANDOM_NAME_COUNT>();
                    let _ = m
                        .transaction()
                        .modify(a, |_k, v| *v += 1)
                        .modify(b, |_k, v| *v += 1)
                        .modify(c, |_k, v| *v += 1)
                        .into_transaction()
                        .execute();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        let total = map.fold(0u64, |_, v| Some(*v), |total, v| total + v);
        assert_eq!(total, THREAD_COUNT * LONG_LOOP * (RANDOM_NAME_COUNT as u64));
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
    }

    #[test]
    fn atomic_transaction_isolation() {
        let map = Arc::new(empty_typed_map::<u64, u64>());
        map.insert(1, 0);
        let map_clone = map.clone();
        let map_clone2 = map.clone();
        let barrier = Arc::new(Barrier::new(2));
        let b1 = barrier.clone();
        let b2 = barrier.clone();

        let h1 = thread::spawn(move || {
            b1.wait();
            for _ in 0..LONG_LOOP {
                let _ = map_clone
                    .transaction()
                    .modify(1, |_k, v| *v += 1)
                    .into_transaction()
                    .execute();
            }
        });

        let h2 = thread::spawn(move || {
            b2.wait();
            for _ in 0..LONG_LOOP {
                let _ = map_clone2
                    .transaction()
                    .modify(1, |_k, v| *v += 1)
                    .into_transaction()
                    .execute();
            }
        });

        h1.join().unwrap();
        h2.join().unwrap();
        assert_eq!(map.get_with(&1, |v| *v), Some(LONG_LOOP * 2));
    }
}
