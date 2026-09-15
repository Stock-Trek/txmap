//! Zipfian skew access benchmarks.
//!
//! Real key-value workloads are rarely uniform: a small number of keys are
//! accessed far more often than the rest. This benchmark samples keys from a
//! Zipfian distribution and measures how `TxMap` behaves as the skew, and
//! therefore the contention, increases.
//!
//! `skew = 0.0` is uniform, `skew = 0.99` approximates the YCSB Zipfian
//! distribution, and larger values concentrate traffic on an even smaller set
//! of hot keys.
//!
//! Run with: `cargo bench --bench zipf`

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hashbrown::HashMap;
use parking_lot::{Mutex, RwLock};
use std::{hint::black_box, thread, time::Duration};
use txmap::prelude::*;

/// Number of entries used by the concurrent benchmarks.
const CONCURRENT_KEYS: u64 = 100_000;
/// Map sizes swept by the single-threaded read benchmark.
const KEY_COUNTS: [u64; 2] = [1_000, 100_000];
/// Zipf skew exponents swept by every benchmark.
const SKEWS: [f64; 4] = [0.0, 0.5, 0.99, 1.29];
/// Point lookups executed by the single-threaded benchmark.
const READ_OPS: usize = 20_000;
/// Operations executed per thread by the concurrent benchmarks.
const OPS_PER_THREAD: usize = 2_000;
/// Thread counts swept by the concurrent benchmarks.
const THREAD_COUNTS: [usize; 3] = [1, 4, 8];

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

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Draws key indices with probability proportional to `1 / (index + 1)^skew`.
struct Zipf {
    cumulative: Vec<f64>,
}

impl Zipf {
    fn new(keys: u64, skew: f64) -> Self {
        let mut cumulative = Vec::with_capacity(keys as usize);
        let mut total = 0.0;
        for rank in 1..=keys {
            total += 1.0 / (rank as f64).powf(skew);
            cumulative.push(total);
        }
        for probability in &mut cumulative {
            *probability /= total;
        }
        Self { cumulative }
    }

    /// Samples a key index; index `0` is the hottest key.
    fn sample(&self, rng: &mut Rng) -> u64 {
        let target = rng.next_f64();
        self.cumulative
            .partition_point(|&probability| probability <= target)
            .min(self.cumulative.len() - 1) as u64
    }
}

/// Builds a reproducible sequence of Zipf-distributed key indices.
fn build_trace(zipf: &Zipf, seed: u64, len: usize) -> Vec<u64> {
    let mut rng = Rng::new(seed);
    (0..len).map(|_| zipf.sample(&mut rng)).collect()
}

/// Spawns one thread per trace and joins them all.
fn run_concurrent<M, F>(map: &M, traces: &[Vec<u64>], run: F)
where
    M: Sync,
    F: Fn(&M, &[u64]) + Sync,
{
    thread::scope(|scope| {
        for trace in traces {
            scope.spawn(|| run(map, trace));
        }
    });
}

fn run_txmap_reads(map: &TxMap<u64, u64>, keys: &[u64]) {
    for key in keys {
        black_box(map.get_with(key, |value| *value));
    }
}

fn run_hashmap_reads(map: &HashMap<u64, u64>, keys: &[u64]) {
    for key in keys {
        black_box(map.get(key));
    }
}

fn run_rwlock_reads(map: &RwLock<HashMap<u64, u64>>, keys: &[u64]) {
    for key in keys {
        black_box(map.read().get(key));
    }
}

fn run_txmap_modifies(map: &TxMap<u64, u64>, keys: &[u64]) {
    for key in keys {
        black_box(map.modify(key, |_, value| *value = value.wrapping_add(1)));
    }
}

fn run_mutex_modifies(map: &Mutex<HashMap<u64, u64>>, keys: &[u64]) {
    for key in keys {
        let mut guard = map.lock();
        if let Some(value) = guard.get_mut(key) {
            *value = value.wrapping_add(1);
            black_box(value);
        }
    }
}

fn single_threaded_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipf/read");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(50);
    group.throughput(Throughput::Elements(READ_OPS as u64));

    for key_count in KEY_COUNTS {
        let txmap: TxMap<u64, u64> = TxMap::new();
        let mut hashmap: HashMap<u64, u64> = HashMap::new();
        for key in 0..key_count {
            txmap.insert(key, key);
            hashmap.insert(key, key);
        }

        for skew in SKEWS {
            let trace = build_trace(&Zipf::new(key_count, skew), 0x5EED, READ_OPS);
            let parameter = format!("keys_{key_count}/skew_{skew}");

            group.bench_with_input(BenchmarkId::new("txmap", &parameter), &trace, |b, trace| {
                b.iter(|| run_txmap_reads(&txmap, trace));
            });
            group.bench_with_input(
                BenchmarkId::new("hashbrown", &parameter),
                &trace,
                |b, trace| b.iter(|| run_hashmap_reads(&hashmap, trace)),
            );
        }
    }
    group.finish();
}

fn concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipf/concurrent_read");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    for skew in SKEWS {
        let zipf = Zipf::new(CONCURRENT_KEYS, skew);
        for threads in THREAD_COUNTS {
            let traces: Vec<Vec<u64>> = (0..threads)
                .map(|thread| build_trace(&zipf, 0xA11CE ^ (thread as u64 + 1), OPS_PER_THREAD))
                .collect();
            group.throughput(Throughput::Elements((threads * OPS_PER_THREAD) as u64));
            let parameter = format!("skew_{skew}/threads_{threads}");

            let txmap: TxMap<u64, u64> = TxMap::new();
            let hashmap: RwLock<HashMap<u64, u64>> = RwLock::new(HashMap::new());
            for key in 0..CONCURRENT_KEYS {
                txmap.insert(key, key);
                hashmap.write().insert(key, key);
            }

            group.bench_with_input(
                BenchmarkId::new("txmap", &parameter),
                &traces,
                |b, traces| {
                    b.iter(|| run_concurrent(&txmap, traces, run_txmap_reads));
                },
            );
            group.bench_with_input(
                BenchmarkId::new("rwlock_hashbrown", &parameter),
                &traces,
                |b, traces| b.iter(|| run_concurrent(&hashmap, traces, run_rwlock_reads)),
            );
        }
    }
    group.finish();
}

fn concurrent_modifies(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipf/concurrent_modify");
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    for skew in SKEWS {
        let zipf = Zipf::new(CONCURRENT_KEYS, skew);
        for threads in THREAD_COUNTS {
            let traces: Vec<Vec<u64>> = (0..threads)
                .map(|thread| build_trace(&zipf, 0xBEEF ^ (thread as u64 + 1), OPS_PER_THREAD))
                .collect();
            group.throughput(Throughput::Elements((threads * OPS_PER_THREAD) as u64));
            let parameter = format!("skew_{skew}/threads_{threads}");

            let txmap: TxMap<u64, u64> = TxMap::new();
            let hashmap: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
            for key in 0..CONCURRENT_KEYS {
                txmap.insert(key, key);
                hashmap.lock().insert(key, key);
            }

            group.bench_with_input(
                BenchmarkId::new("txmap", &parameter),
                &traces,
                |b, traces| {
                    b.iter(|| run_concurrent(&txmap, traces, run_txmap_modifies));
                },
            );
            group.bench_with_input(
                BenchmarkId::new("mutex_hashbrown", &parameter),
                &traces,
                |b, traces| b.iter(|| run_concurrent(&hashmap, traces, run_mutex_modifies)),
            );
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    single_threaded_reads,
    concurrent_reads,
    concurrent_modifies
);
criterion_main!(benches);
