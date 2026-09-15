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
//! Parallel iterators acquire a leaf lock on every shard up front and hold
//! all of them until iteration completes, so the map is observed as a
//! consistent snapshot and cannot be mutated while a parallel iteration is
//! running.

use crate::{custodian::Custodian, new_types::ShardIndex, tx_map::TxMap};
use hashbrown::hash_table::Iter as ShardIter;
use rayon::iter::plumbing::{Folder, UnindexedConsumer, UnindexedProducer, bridge_unindexed};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::hash::BuildHasher;

/// Parallel iterator over all key-value pairs in a [`TxMap`].
///
/// Created by [`TxMap::par_iter`] or by calling `into_par_iter` on a
/// `&TxMap` / `&mut TxMap`. Acquires a leaf lock on every shard up front
/// and holds all of them for the duration of iteration, so the map is a
/// consistent snapshot while it runs.
pub struct ParIter<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    pub(crate) custodian: &'a Custodian<K, V>,
}

impl<'a, K, V> Clone for ParIter<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    fn clone(&self) -> Self {
        Self {
            custodian: self.custodian,
        }
    }
}

impl<'a, K, V> ParallelIterator for ParIter<'a, K, V>
where
    K: Sync,
    V: Sync,
    Custodian<K, V>: Sync,
{
    type Item = (&'a K, &'a V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        // Lock every active leaf and hold all of them until the parallel
        // iteration below completes, giving a consistent snapshot.
        let (_topology, topology_mask) = self.custodian.acquire_all();
        let ids = self.custodian.active_ids_in(topology_mask);
        let mut shard_iters: Vec<ShardIter<'a, (K, V)>> = Vec::with_capacity(ids.len());
        for id in ids {
            let guard = self.custodian.read_guard_at(ShardIndex(id));
            // SAFETY: `ShardIter` stores only raw pointers into the shard's
            // heap-allocated buckets plus a `PhantomData` marker; the lifetime
            // is not tracked at runtime. The topology mask keeps the shard data
            // alive and immutable for the entire `bridge_unindexed` call below
            // (it is dropped only after it returns), so the iterators can
            // never outlive the data they reference.
            let iter: ShardIter<'a, (K, V)> = unsafe { std::mem::transmute(guard.iter()) };
            shard_iters.push(iter);
        }
        bridge_unindexed(ParIterProducer { shard_iters }, consumer)
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
/// Created by [`TxMap::par_keys`]. Acquires leaf locks on all shards for
/// the duration of iteration.
pub struct ParKeys<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    par_iter: ParIter<'a, K, V>,
}

impl<'a, K, V> ParallelIterator for ParKeys<'a, K, V>
where
    K: Sync,
    V: Sync,
    Custodian<K, V>: Sync,
{
    type Item = &'a K;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.par_iter.map(|(key, _)| key).drive_unindexed(consumer)
    }
}

/// Parallel iterator over all the values in a [`TxMap`].
///
/// Created by [`TxMap::par_values`]. Acquires leaf locks on all shards for
/// the duration of iteration.
pub struct ParValues<'a, K, V>
where
    K: 'a,
    V: 'a,
{
    par_iter: ParIter<'a, K, V>,
}

impl<'a, K, V> ParallelIterator for ParValues<'a, K, V>
where
    K: Sync,
    V: Sync,
    Custodian<K, V>: Sync,
{
    type Item = &'a V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.par_iter
            .map(|(_, value)| value)
            .drive_unindexed(consumer)
    }
}

// `Custodian` is `pub(crate)`, but it is the type whose shards these parallel
// iterators traverse; the map's hasher (`S`) plays no part in iteration.
// Users never need to name `Custodian` to use these methods, so bounding on
// it here does not leak into the public API.
#[allow(private_bounds)]
impl<K, V, S> TxMap<K, V, S>
where
    K: Sync,
    V: Sync,
    S: BuildHasher,
    Custodian<K, V>: Sync,
{
    /// Returns a parallel iterator over all key-value pairs.
    ///
    /// Acquires leaf locks on all shards for the duration of iteration.
    #[must_use]
    pub fn par_iter(&self) -> ParIter<'_, K, V> {
        ParIter {
            custodian: &self.custodian,
        }
    }

    /// Returns a parallel iterator over all the keys.
    ///
    /// Acquires leaf locks on all shards for the duration of iteration.
    #[must_use]
    pub fn par_keys(&self) -> ParKeys<'_, K, V> {
        ParKeys {
            par_iter: self.par_iter(),
        }
    }

    /// Returns a parallel iterator over all the values.
    ///
    /// Acquires leaf locks on all shards for the duration of iteration.
    #[must_use]
    pub fn par_values(&self) -> ParValues<'_, K, V> {
        ParValues {
            par_iter: self.par_iter(),
        }
    }
}

impl<'a, K, V, S> IntoParallelIterator for &'a TxMap<K, V, S>
where
    K: Sync,
    V: Sync,
    S: BuildHasher,
    Custodian<K, V>: Sync,
{
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter {
            custodian: &self.custodian,
        }
    }
}

impl<'a, K, V, S> IntoParallelIterator for &'a mut TxMap<K, V, S>
where
    K: Sync,
    V: Sync,
    S: BuildHasher,
    Custodian<K, V>: Sync,
{
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter {
            custodian: &self.custodian,
        }
    }
}

impl<K, V, S> IntoParallelIterator for TxMap<K, V, S>
where
    K: Send,
    V: Send,
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
