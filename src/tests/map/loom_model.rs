//! Loom model of the lock-free acquire / version-check protocol.
//!
//! This is a self-contained model rather than a direct test of [`ShardMap`],
//! so it can run with loom's instrumented atomics without threading a
//! `cfg(loom)` atomic abstraction through the crate. It checks the core
//! invariants of the protocol:
//!
//! * the all-or-nothing mask gives mutual exclusion over the leaves, and
//! * a transaction that observes unchanged versions after acquiring the mask
//!   sees a stable topology, so a concurrent version bump either makes the
//!   transaction retry or is excluded by the mask — never silently lost.
//!
//! Run with `RUSTFLAGS="--cfg loom" cargo test --lib map::loom_model`.

use loom::sync::Arc;
use loom::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use loom::thread;

const LEAVES: usize = 2;
const ALL_LEAVES: u8 = 0b11;

struct Arbiter {
    locked: AtomicU8,
    active: AtomicU32,
    max_active: AtomicU32,
    versions: [AtomicU32; LEAVES],
    counts: [AtomicU32; LEAVES],
}

impl Arbiter {
    fn new() -> Self {
        Self {
            locked: AtomicU8::new(0),
            active: AtomicU32::new(0),
            max_active: AtomicU32::new(0),
            versions: std::array::from_fn(|_| AtomicU32::new(0)),
            counts: std::array::from_fn(|_| AtomicU32::new(0)),
        }
    }

    fn try_acquire(&self, needed: u8) -> bool {
        let mut current = self.locked.load(Ordering::Acquire);
        loop {
            if current & needed != 0 {
                return false;
            }
            match self.locked.compare_exchange_weak(
                current,
                current | needed,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(observed) => current = observed,
            }
        }
    }

    fn release(&self, mask: u8) {
        self.locked.fetch_and(!mask, Ordering::Release);
    }

    fn enter_critical(&self) {
        let active = self.active.fetch_add(1, Ordering::AcqRel) + 1;
        self.max_active.fetch_max(active, Ordering::AcqRel);
    }

    fn exit_critical(&self) {
        self.active.fetch_sub(1, Ordering::Release);
    }

    fn versions(&self) -> [u32; LEAVES] {
        std::array::from_fn(|i| self.versions[i].load(Ordering::Acquire))
    }

    fn snapshot_matches(&self, snapshot: &[u32; LEAVES]) -> bool {
        (0..LEAVES).all(|i| self.versions[i].load(Ordering::Acquire) == snapshot[i])
    }
}

/// A single transaction attempt: snapshot versions, acquire all leaves,
/// re-check, and only then update. Returns whether it committed.
fn transaction(state: &Arbiter) -> bool {
    let snapshot = state.versions();
    if !state.try_acquire(ALL_LEAVES) {
        return false;
    }
    if !state.snapshot_matches(&snapshot) {
        state.release(ALL_LEAVES);
        return false;
    }
    state.enter_critical();
    for count in &state.counts {
        count.fetch_add(1, Ordering::Relaxed);
    }
    state.exit_critical();
    state.release(ALL_LEAVES);
    true
}

#[test]
fn versions_and_mask_preserve_atomicity() {
    loom::model(|| {
        let state = Arc::new(Arbiter::new());

        let transactor = state.clone();
        let transactor_handle = thread::spawn(move || transaction(&transactor));

        // A topology change bumps every version while holding the mask.
        let changer = state.clone();
        let changer_handle = thread::spawn(move || {
            if changer.try_acquire(ALL_LEAVES) {
                changer.enter_critical();
                for version in &changer.versions {
                    version.fetch_add(1, Ordering::AcqRel);
                }
                changer.exit_critical();
                changer.release(ALL_LEAVES);
            }
        });

        let committed = transactor_handle.join().unwrap();
        changer_handle.join().unwrap();

        // Mutual exclusion: the two critical sections never overlapped.
        assert!(state.max_active.load(Ordering::Acquire) <= 1);
        // The mask must be fully released.
        assert_eq!(state.locked.load(Ordering::Acquire), 0);

        // The transaction either committed on every leaf or none: a partial
        // write would mean another actor was mutating concurrently.
        let total: u32 = state
            .counts
            .iter()
            .map(|count| count.load(Ordering::Relaxed))
            .sum();
        if committed {
            assert_eq!(total, LEAVES as u32);
        } else {
            assert_eq!(total, 0);
        }
    });
}
