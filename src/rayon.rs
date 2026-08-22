//! Rayon support, gated behind the `rayon` feature.
//!
//! Provides parallel iterators over a [`TxMap`], mirroring the serial
//! iterators in [`crate::iter`]:
//!
//! - [`TxMap::par_iter`] / [`IntoParallelIterator`] for `&TxMap` and
//!   `&mut TxMap` yield `(&K, &V)`.
//! - [`TxMap::par_keys`] and [`TxMap::par_values`] yield `&K` and `&V`.
//! - [`IntoParallelIterator`] for owned `TxMap` yields `(K, V)` (eager,
//!   matching the serial owned `IntoIterator`).
//!
//! Parallel iterators acquire a read guard on every shard up front and hold
//! all guards until iteration completes, so the map is observed as a
//! consistent snapshot and cannot be mutated while a parallel iteration is
//! running.

use crate::{
    lock_policies::lock_policy::LockPolicy, new_types::ShardIndex, shard::Shard, tx_map::TxMap,
};
use hashbrown::hash_table::Iter as ShardIter;
use rayon::iter::plumbing::{Folder, UnindexedConsumer, UnindexedProducer, bridge_unindexed};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hash::BuildHasher;

/// Parallel iterator over all key-value pairs in a [`TxMap`].
///
/// Created by [`TxMap::par_iter`] or by calling `into_par_iter` on a
/// `&TxMap` / `&mut TxMap`. Acquires a read guard on every shard up front
/// and holds all guards for the duration of iteration, so the map is a
/// consistent snapshot while it runs.
pub struct ParIter<'a, K, V, L, S>
where
    K: 'a,
    V: 'a,
    L: LockPolicy,
    S: BuildHasher,
{
    pub(crate) map: &'a TxMap<K, V, L, S>,
}

impl<'a, K, V, L, S> Clone for ParIter<'a, K, V, L, S>
where
    K: 'a,
    V: 'a,
    L: LockPolicy,
    S: BuildHasher,
{
    fn clone(&self) -> Self {
        Self { map: self.map }
    }
}

impl<'a, K, V, L, S> ParallelIterator for ParIter<'a, K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    type Item = (&'a K, &'a V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let shard_count = self.map.shard_count.0 as usize;
        // Acquire a read guard on every shard and hold all of them until the
        // parallel iteration below completes, giving a consistent snapshot.
        let mut guards: Vec<L::ReadGuard<'_, Shard<K, V>>> = Vec::with_capacity(shard_count);
        let mut shard_iters: Vec<ShardIter<'a, (K, V)>> = Vec::with_capacity(shard_count);
        for shard_index in 0..shard_count {
            let guard = self
                .map
                .custodian
                .read_guard_at(ShardIndex(shard_index as u8));
            // SAFETY: `ShardIter` stores only raw pointers into the shard's
            // heap-allocated buckets plus a `PhantomData` marker; the lifetime
            // is not tracked at runtime. The read guards keep the shard data
            // alive and immutable for the entire `bridge_unindexed` call below
            // (they are dropped only after it returns), so the iterators can
            // never outlive the data they reference.
            let iter: ShardIter<'a, (K, V)> = unsafe { std::mem::transmute(guard.iter()) };
            guards.push(guard);
            shard_iters.push(iter);
        }
        let result = bridge_unindexed(ParIterProducer { shard_iters }, consumer);
        drop(guards);
        result
    }
}

/// Producer that yields entries from a list of shard iterators, splitting
/// across shard boundaries.
struct ParIterProducer<'a, K, V> {
    shard_iters: Vec<ShardIter<'a, (K, V)>>,
}

impl<'a, K, V> UnindexedProducer for ParIterProducer<'a, K, V>
where
    K: Sync,
    V: Sync,
{
    type Item = (&'a K, &'a V);

    fn split(mut self) -> (Self, Option<Self>) {
        let len = self.shard_iters.len();
        if len <= 1 {
            (self, None)
        } else {
            let mid = len / 2;
            let right = self.shard_iters.split_off(mid);
            (
                Self {
                    shard_iters: self.shard_iters,
                },
                Some(Self { shard_iters: right }),
            )
        }
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        folder.consume_iter(
            self.shard_iters
                .into_iter()
                .flatten()
                .map(|entry| (&entry.0, &entry.1)),
        )
    }
}

/// Parallel iterator over all the keys in a [`TxMap`].
///
/// Created by [`TxMap::par_keys`]. Acquires read guards on all shards for
/// the duration of iteration.
pub struct ParKeys<'a, K, V, L, S>
where
    K: 'a,
    V: 'a,
    L: LockPolicy,
    S: BuildHasher,
{
    inner: ParIter<'a, K, V, L, S>,
}

impl<'a, K, V, L, S> ParallelIterator for ParKeys<'a, K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    type Item = &'a K;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.inner.map(|(key, _)| key).drive_unindexed(consumer)
    }
}

/// Parallel iterator over all the values in a [`TxMap`].
///
/// Created by [`TxMap::par_values`]. Acquires read guards on all shards for
/// the duration of iteration.
pub struct ParValues<'a, K, V, L, S>
where
    K: 'a,
    V: 'a,
    L: LockPolicy,
    S: BuildHasher,
{
    inner: ParIter<'a, K, V, L, S>,
}

impl<'a, K, V, L, S> ParallelIterator for ParValues<'a, K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    type Item = &'a V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.inner.map(|(_, value)| value).drive_unindexed(consumer)
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    /// Returns a parallel iterator over all key-value pairs.
    ///
    /// Acquires read guards on all shards for the duration of iteration.
    #[must_use]
    pub fn par_iter(&self) -> ParIter<'_, K, V, L, S> {
        ParIter { map: self }
    }

    /// Returns a parallel iterator over all the keys.
    ///
    /// Acquires read guards on all shards for the duration of iteration.
    #[must_use]
    pub fn par_keys(&self) -> ParKeys<'_, K, V, L, S> {
        ParKeys {
            inner: self.par_iter(),
        }
    }

    /// Returns a parallel iterator over all the values.
    ///
    /// Acquires read guards on all shards for the duration of iteration.
    #[must_use]
    pub fn par_values(&self) -> ParValues<'_, K, V, L, S> {
        ParValues {
            inner: self.par_iter(),
        }
    }
}

impl<'a, K, V, L, S> IntoParallelIterator for &'a TxMap<K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V, L, S>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter { map: self }
    }
}

impl<'a, K, V, L, S> IntoParallelIterator for &'a mut TxMap<K, V, L, S>
where
    K: Sync,
    V: Sync,
    L: LockPolicy,
    S: BuildHasher,
    TxMap<K, V, L, S>: Sync,
{
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V, L, S>;

    fn into_par_iter(self) -> Self::Iter {
        let map: &'a TxMap<K, V, L, S> = self;
        ParIter { map }
    }
}

impl<K, V, L, S> IntoParallelIterator for TxMap<K, V, L, S>
where
    K: Send,
    V: Send,
    L: LockPolicy,
    S: BuildHasher,
{
    type Item = (K, V);
    type Iter = rayon::vec::IntoIter<(K, V)>;

    /// Consumes the map and iterates over its entries in parallel.
    ///
    /// Mirrors the eager owned `IntoIterator`: all entries are drained into
    /// a buffer before the map is dropped, then iterated in parallel.
    fn into_par_iter(self) -> Self::Iter {
        self.drain().collect::<Vec<(K, V)>>().into_par_iter()
    }
}
