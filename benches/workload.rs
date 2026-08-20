//! Concurrent workload benchmarks.
//!
//! Simulates realistic concurrent access patterns — read-heavy caches,
//! write-heavy ingestion, counter-style read-modify-write, and atomic
//! two-key transfers — under two contention profiles (partitioned keys
//! and a shared hot-key set) and thread counts from 1 to 6.
//!
//! Each workload is compared against `RwLock<HashMap>` and
//! `Mutex<HashMap>` baselines to show how `TxMap`'s shard-level locking
//! scales under contention.
//!
//! Run with: `cargo bench --bench workload`

use criterion::{Criterion, criterion_group, criterion_main};
use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};
use std::{hint::black_box, thread, time::Duration};
use txmap::prelude::*;

/// Operations executed per thread per measured iteration.
const OPS_PER_THREAD: usize = 2_000;
/// Keys owned by each thread in partitioned mode.
const PARTITION_KEYS: u64 = 1_024;
/// Shared hot-key set size in contended mode (sized so threads frequently
/// collide on the same shard, exercising lock contention).
const HOT_KEYS: u64 = 64;
/// Thread counts swept for every workload.
const THREAD_COUNTS: [usize; 6] = [1, 2, 3, 4, 5, 6];

#[derive(Clone, Copy)]
enum Workload {
    /// Cache-like: 85% gets, 10% inserts, 5% removes.
    ReadHeavy,
    /// Ingest-like: 40% gets, 60% inserts.
    WriteHeavy,
    /// Counter-like: 100% in-place increments.
    ReadModifyWrite,
    /// Bank-like: 100% atomic two-key transfers.
    Transfer,
}

impl Workload {
    const ALL: [Workload; 4] = [
        Workload::ReadHeavy,
        Workload::WriteHeavy,
        Workload::ReadModifyWrite,
        Workload::Transfer,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Workload::ReadHeavy => "read_heavy",
            Workload::WriteHeavy => "write_heavy",
            Workload::ReadModifyWrite => "read_modify_write",
            Workload::Transfer => "transfer",
        }
    }
}

#[derive(Clone, Copy)]
enum KeyMode {
    /// Each thread only touches its own private key range.
    Partitioned,
    /// All threads hammer the same small hot-key set.
    Contended,
}

impl KeyMode {
    const ALL: [KeyMode; 2] = [KeyMode::Partitioned, KeyMode::Contended];

    fn as_str(self) -> &'static str {
        match self {
            KeyMode::Partitioned => "partitioned",
            KeyMode::Contended => "contended",
        }
    }
}

/// A single operation in a generated workload trace.
#[derive(Clone, Copy)]
enum Op {
    Get(u64),
    Insert(u64, u64),
    Remove(u64),
    Update(u64),
    Transfer(u64, u64),
}

/// Deterministic xorshift64* PRNG so traces are reproducible across runs.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Builds one thread's operation trace for the given workload and key mode.
fn build_trace(workload: Workload, mode: KeyMode, thread_id: usize) -> Vec<Op> {
    let seed = 0x9E37_79B9_7F4A_7C15 ^ ((thread_id as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    let mut rng = Rng::new(seed);
    (0..OPS_PER_THREAD)
        .map(|_| {
            let key = |rng: &mut Rng| match mode {
                KeyMode::Partitioned => {
                    thread_id as u64 * PARTITION_KEYS + rng.below(PARTITION_KEYS)
                }
                KeyMode::Contended => rng.below(HOT_KEYS),
            };
            let roll = rng.below(100);
            match workload {
                Workload::ReadHeavy => {
                    if roll < 85 {
                        Op::Get(key(&mut rng))
                    } else if roll < 95 {
                        Op::Insert(key(&mut rng), rng.next_u64())
                    } else {
                        Op::Remove(key(&mut rng))
                    }
                }
                Workload::WriteHeavy => {
                    if roll < 40 {
                        Op::Get(key(&mut rng))
                    } else {
                        Op::Insert(key(&mut rng), rng.next_u64())
                    }
                }
                Workload::ReadModifyWrite => Op::Update(key(&mut rng)),
                Workload::Transfer => {
                    let from = key(&mut rng);
                    let mut to = key(&mut rng);
                    while to == from {
                        to = key(&mut rng);
                    }
                    Op::Transfer(from, to)
                }
            }
        })
        .collect()
}

/// Executes a trace against `TxMap`.
fn run_txmap(map: &TxMap<u64, u64>, ops: &[Op]) {
    for op in ops {
        match *op {
            Op::Get(k) => {
                black_box(map.get_with(&k, |v| *v));
            }
            Op::Insert(k, v) => {
                black_box(map.insert(k, v));
            }
            Op::Remove(k) => {
                black_box(map.remove(&k));
            }
            Op::Update(k) => {
                black_box(map.modify(&k, |_, v| *v += 1));
            }
            Op::Transfer(a, b) => {
                black_box(
                    map.immediate_tx::<()>()
                        .modify(a, |_, v, _| *v = v.wrapping_add(1))
                        .modify(b, |_, v, _| *v = v.wrapping_sub(1))
                        .execute(),
                );
            }
        }
    }
}

/// Executes a trace against a `parking_lot::RwLock<HashMap>` baseline.
fn run_rwlock(map: &RwLock<HashMap<u64, u64>>, ops: &[Op]) {
    for op in ops {
        match *op {
            Op::Get(k) => {
                black_box(map.read().get(&k).copied());
            }
            Op::Insert(k, v) => {
                black_box(map.write().insert(k, v));
            }
            Op::Remove(k) => {
                black_box(map.write().remove(&k));
            }
            Op::Update(k) => {
                let mut guard = map.write();
                if let Some(v) = guard.get_mut(&k) {
                    *v = v.wrapping_add(1);
                }
            }
            Op::Transfer(a, b) => {
                let mut guard = map.write();
                if let Some(v) = guard.get_mut(&a) {
                    *v = v.wrapping_add(1);
                }
                if let Some(v) = guard.get_mut(&b) {
                    *v = v.wrapping_sub(1);
                }
            }
        }
    }
}

/// Executes a trace against a `parking_lot::Mutex<HashMap>` baseline.
fn run_mutex(map: &Mutex<HashMap<u64, u64>>, ops: &[Op]) {
    for op in ops {
        match *op {
            Op::Get(k) => {
                black_box(map.lock().get(&k).copied());
            }
            Op::Insert(k, v) => {
                black_box(map.lock().insert(k, v));
            }
            Op::Remove(k) => {
                black_box(map.lock().remove(&k));
            }
            Op::Update(k) => {
                let mut guard = map.lock();
                if let Some(v) = guard.get_mut(&k) {
                    *v = v.wrapping_add(1);
                }
            }
            Op::Transfer(a, b) => {
                let mut guard = map.lock();
                if let Some(v) = guard.get_mut(&a) {
                    *v = v.wrapping_add(1);
                }
                if let Some(v) = guard.get_mut(&b) {
                    *v = v.wrapping_sub(1);
                }
            }
        }
    }
}

/// Spawns one thread per trace and joins them all.
fn run_concurrent<M, F>(map: &M, traces: &[Vec<Op>], run: F)
where
    M: Sync,
    F: Fn(&M, &[Op]) + Sync,
{
    thread::scope(|scope| {
        for trace in traces {
            scope.spawn(|| run(map, trace));
        }
    });
}

fn workloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("workload");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    group.sample_size(50);

    for workload in Workload::ALL {
        for mode in KeyMode::ALL {
            for &threads in &THREAD_COUNTS {
                let traces: Vec<Vec<Op>> = (0..threads)
                    .map(|t| build_trace(workload, mode, t))
                    .collect();
                let key_count = match mode {
                    KeyMode::Partitioned => threads as u64 * PARTITION_KEYS,
                    KeyMode::Contended => HOT_KEYS,
                };
                let prefix = format!("{}/{}/", workload.as_str(), mode.as_str());
                let throughput =
                    || criterion::Throughput::Elements((threads * OPS_PER_THREAD) as u64);

                // TxMap (default 32 shards).
                let map = TxMap::<u64, u64>::new();
                for k in 0..key_count {
                    map.insert(k, 0);
                }
                group.throughput(throughput());
                group.bench_function(format!("{}txmap/threads_{}", prefix, threads), |b| {
                    b.iter(|| run_concurrent(&map, &traces, run_txmap))
                });

                // RwLock<HashMap> baseline.
                let map =
                    RwLock::<HashMap<u64, u64>>::new((0..key_count).map(|k| (k, 0)).collect());
                group.throughput(throughput());
                group.bench_function(format!("{}rwlock/threads_{}", prefix, threads), |b| {
                    b.iter(|| run_concurrent(&map, &traces, run_rwlock))
                });

                // Mutex<HashMap> baseline.
                let map = Mutex::<HashMap<u64, u64>>::new((0..key_count).map(|k| (k, 0)).collect());
                group.throughput(throughput());
                group.bench_function(format!("{}mutex/threads_{}", prefix, threads), |b| {
                    b.iter(|| run_concurrent(&map, &traces, run_mutex))
                });
            }
        }
    }
}

criterion_group!(benches, workloads);
criterion_main!(benches);
