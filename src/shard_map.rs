//! Leaf storage with lock-free routing.
//!
//! [`ShardMap`] owns a fixed array of leaf slots, a routing trie and the
//! atomic masks used by the transaction lock protocol. Leaf slots never
//! move; splits and merges only change routing, versions and the set of
//! active leaf ids.

use crate::trie::RoutingTrie;
use portable_atomic::AtomicU128;
use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};

/// Maximum number of leaves a [`ShardMap`] can hold.
pub(crate) const MAX_LEAVES: usize = 128;

/// Fixed leaf storage plus the routing and synchronisation metadata.
pub(crate) struct ShardMap<T> {
    /// Fixed leaf storage. A slot is initialised exactly when its bit is set
    /// in `active_mask`. Slots never move.
    leaves: [UnsafeCell<MaybeUninit<T>>; MAX_LEAVES],
    /// Bit `i` is set when leaf `i` exists in the routing trie.
    active_mask: AtomicU128,
    /// Bit `i` is set once leaf `i` has been fully initialised.
    ///
    /// A split sets the bit in `active_mask` (to reserve the id) before the
    /// leaf is written; `ready_mask` closes that window so snapshotting code
    /// such as [`ShardMap::active_ids`] never observes an uninitialised slot.
    ready_mask: AtomicU128,
    /// Bit `i` is set while leaf `i` is held by a transaction.
    locked_mask: AtomicU128,
    /// Bumped whenever the routing for leaf `i` changes.
    versions: [AtomicU32; MAX_LEAVES],
    /// Counts how often an acquisition had to back off because leaf `i` was
    /// already locked. Used to detect hot leaves and drive adaptive splits.
    contention: [AtomicU32; MAX_LEAVES],
    /// Hash prefix to leaf id mapping.
    pub(crate) routing: RoutingTrie,
    /// Maximum number of active leaves (the configured shard count).
    budget: u32,
}

// SAFETY: access to the unsynchronised leaf slots is coordinated by
// `locked_mask` (or by exclusive ownership during construction/destruction).
// `T: Send + Sync` therefore makes `ShardMap<T>` `Send + Sync`.
unsafe impl<T: Send> Send for ShardMap<T> {}
unsafe impl<T: Send + Sync> Sync for ShardMap<T> {}

impl<T> ShardMap<T> {
    /// Creates a map with a single active leaf holding `initial`.
    pub(crate) fn new(initial: T, budget: u32) -> Self {
        debug_assert!(
            AtomicU128::is_lock_free(),
            "this target does not provide lock-free 128-bit atomics"
        );
        let leaves = std::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit()));
        let map = Self {
            leaves,
            active_mask: AtomicU128::new(0),
            ready_mask: AtomicU128::new(0),
            locked_mask: AtomicU128::new(0),
            versions: std::array::from_fn(|_| AtomicU32::new(0)),
            contention: std::array::from_fn(|_| AtomicU32::new(0)),
            routing: RoutingTrie::new(0),
            budget: budget.min(MAX_LEAVES as u32),
        };
        // SAFETY: `&map` is exclusively owned and leaf 0 is not active yet.
        unsafe { map.init_leaf(0, initial) };
        map
    }

    /// Routes a hash to a leaf id.
    #[inline]
    pub(crate) fn route(&self, hash: u64) -> u8 {
        self.routing.route(hash)
    }

    /// Number of active leaves.
    #[inline]
    pub(crate) fn leaf_count(&self) -> u32 {
        self.active_mask.load(Ordering::Acquire).count_ones()
    }

    /// Maximum number of leaves this map may hold.
    #[inline]
    pub(crate) fn budget(&self) -> u32 {
        self.budget
    }

    /// Snapshot of the active leaf ids.
    pub(crate) fn active_ids(&self) -> Vec<u8> {
        let mut mask =
            self.active_mask.load(Ordering::Acquire) & self.ready_mask.load(Ordering::Acquire);
        let mut ids = Vec::with_capacity(mask.count_ones() as usize);
        while mask != 0 {
            ids.push(mask.trailing_zeros() as u8);
            mask &= mask - 1;
        }
        ids
    }

    /// Current version of leaf `id`.
    #[inline]
    pub(crate) fn version(&self, id: u8) -> u32 {
        self.versions[id as usize].load(Ordering::Acquire)
    }

    /// Bumps the version of leaf `id`.
    #[inline]
    pub(crate) fn bump_version(&self, id: u8) {
        self.versions[id as usize].fetch_add(1, Ordering::AcqRel);
    }

    /// Number of times an acquisition backed off because leaf `id` was held.
    #[inline]
    pub(crate) fn contention(&self, id: u8) -> u32 {
        self.contention[id as usize].load(Ordering::Relaxed)
    }

    /// Clears the contention counter for leaf `id`.
    #[inline]
    pub(crate) fn reset_contention(&self, id: u8) {
        self.contention[id as usize].store(0, Ordering::Relaxed);
    }

    /// Acquires every leaf named in `needed`, all-or-nothing.
    pub(crate) fn acquire(&self, needed: u128) {
        if needed == 0 {
            return;
        }
        let mut spins = 0u32;
        // Only record a leaf once per acquisition, even if the CAS loop
        // retries: a single transaction waiting on a leaf is one contention
        // event, not one per spin.
        let mut recorded = 0u128;
        loop {
            let cur = self.locked_mask.load(Ordering::Acquire);
            let conflict = cur & needed;
            if conflict == 0 {
                if self
                    .locked_mask
                    .compare_exchange_weak(cur, cur | needed, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    return;
                }
            } else {
                let mut bits = conflict & !recorded;
                recorded |= conflict;
                while bits != 0 {
                    let id = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    self.contention[id].fetch_add(1, Ordering::Relaxed);
                }
            }
            backoff(&mut spins);
        }
    }

    /// Acquires `needed` and returns a guard that releases it on drop.
    pub(crate) fn acquire_guard(&self, needed: u128) -> MaskGuard<'_, T> {
        self.acquire(needed);
        MaskGuard::new(self, needed)
    }

    /// Acquires every currently active leaf and returns a guard.
    pub(crate) fn acquire_all_guard(&self) -> MaskGuard<'_, T> {
        let mask = self.acquire_all();
        MaskGuard::new(self, mask)
    }

    /// Acquires every leaf named in `needed`, then verifies that the active
    /// set did not change underneath us.
    pub(crate) fn acquire_all(&self) -> u128 {
        let mut spins = 0u32;
        loop {
            let needed = self.active_mask.load(Ordering::Acquire);
            self.acquire(needed);
            if self.active_mask.load(Ordering::Acquire) == needed {
                return needed;
            }
            self.release(needed);
            backoff(&mut spins);
        }
    }

    /// Releases every leaf named in `needed`.
    #[inline]
    pub(crate) fn release(&self, needed: u128) {
        self.locked_mask.fetch_and(!needed, Ordering::Release);
    }

    /// Reserves `count` unused leaf ids. Returns `None` when the budget is
    /// exhausted.
    pub(crate) fn allocate_ids(&self, count: u32) -> Option<[u8; 8]> {
        debug_assert!(count > 0 && count <= 8);
        let mut spins = 0u32;
        loop {
            let cur = self.active_mask.load(Ordering::Acquire);
            if cur.count_ones() + count > self.budget {
                return None;
            }
            let free = !cur;
            if free.count_ones() < count {
                return None;
            }
            let mut ids = [0u8; 8];
            let mut remaining = free;
            for id in ids.iter_mut().take(count as usize) {
                *id = remaining.trailing_zeros() as u8;
                remaining &= remaining - 1;
            }
            let mut new_mask = cur;
            for &id in ids.iter().take(count as usize) {
                new_mask |= 1u128 << id;
            }
            if self
                .active_mask
                .compare_exchange_weak(cur, new_mask, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Some(ids);
            }
            backoff(&mut spins);
        }
    }

    /// Frees the given leaf ids.
    pub(crate) fn free_ids(&self, ids: &[u8]) {
        let mut mask = 0u128;
        for &id in ids {
            mask |= 1u128 << id;
        }
        self.active_mask.fetch_and(!mask, Ordering::Release);
    }

    /// Whether leaf `id` has been initialised and not torn down.
    #[inline]
    pub(crate) fn is_ready(&self, id: u8) -> bool {
        self.ready_mask.load(Ordering::Acquire) & (1u128 << id) != 0
    }

    /// Initialises leaf `id` and marks it active.
    ///
    /// # Safety
    ///
    /// The caller must hold leaf `id` exclusively (or own the map), and the
    /// slot must not already be initialised.
    pub(crate) unsafe fn init_leaf(&self, id: u8, value: T) {
        unsafe {
            (*self.leaves[id as usize].get()).write(value);
        }
        self.active_mask.fetch_or(1u128 << id, Ordering::Release);
        self.ready_mask.fetch_or(1u128 << id, Ordering::Release);
    }

    /// Moves the value out of leaf `id` and marks it inactive.
    ///
    /// # Safety
    ///
    /// The caller must hold leaf `id` exclusively, and the slot must be
    /// initialised.
    pub(crate) unsafe fn take_leaf(&self, id: u8) -> T {
        self.active_mask
            .fetch_and(!(1u128 << id), Ordering::Release);
        self.ready_mask.fetch_and(!(1u128 << id), Ordering::Release);
        unsafe { (*self.leaves[id as usize].get()).assume_init_read() }
    }

    /// Returns a reference to leaf `id`.
    ///
    /// # Safety
    ///
    /// The caller must hold leaf `id` and the slot must be initialised.
    #[inline]
    pub(crate) unsafe fn leaf_ref(&self, id: u8) -> &T {
        unsafe { (*self.leaves[id as usize].get()).assume_init_ref() }
    }

    /// Returns a mutable reference to leaf `id`.
    ///
    /// # Safety
    ///
    /// The caller must hold leaf `id` exclusively and the slot must be
    /// initialised.
    #[allow(clippy::mut_from_ref)]
    #[inline]
    pub(crate) unsafe fn leaf_mut(&self, id: u8) -> &mut T {
        unsafe { (*self.leaves[id as usize].get()).assume_init_mut() }
    }
}

impl<T> Drop for ShardMap<T> {
    fn drop(&mut self) {
        // Empty but reused slots stay initialised, so every ready slot must
        // be dropped, not just the currently active ones.
        let mut mask = *self.ready_mask.get_mut();
        while mask != 0 {
            let id = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            // SAFETY: `&mut self` guarantees exclusive access.
            unsafe { (*self.leaves[id].get()).assume_init_drop() };
        }
    }
}

/// Exponential backoff used by the CAS loops.
#[inline]
fn backoff(spins: &mut u32) {
    if *spins < 6 {
        for _ in 0..(1 << *spins) {
            spin_loop();
        }
    } else if *spins < 10 {
        std::thread::yield_now();
    } else {
        // Park instead of spinning so a long wait does not burn a core.
        std::thread::park_timeout(std::time::Duration::from_micros(50));
    }
    *spins = spins.saturating_add(1);
}

/// RAII guard that releases a leaf mask on drop.
pub(crate) struct MaskGuard<'a, T> {
    map: &'a ShardMap<T>,
    mask: u128,
}

impl<'a, T> MaskGuard<'a, T> {
    pub(crate) fn new(map: &'a ShardMap<T>, mask: u128) -> Self {
        Self { map, mask }
    }
}

impl<T> Drop for MaskGuard<'_, T> {
    fn drop(&mut self) {
        self.map.release(self.mask);
    }
}
