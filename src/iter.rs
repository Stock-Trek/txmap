use crate::{
    custodian::Custodian, lock_policies::lock_policy::LockPolicy, new_types::ShardIndex,
    shard::Shard, tx_map::TxMap,
};
use hashbrown::hash_table::{Drain as ShardDrain, Iter as ShardIter};
use std::hash::Hash;

/// An iterator over all key-value pairs in a [`TxMap`].
///
/// Read guards are acquired lazily, one shard at a time, as iteration
/// progresses. Guards for shards already visited are held until the
/// iterator is dropped, so entries yielded remain valid for the lifetime
/// of the iterator.
pub struct Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    /// The shard custodian, used to acquire read guards lazily.
    pub(crate) custodian: &'a Custodian<K, V, L>,
    /// Read guards keeping every shard locked (and alive) for `'a`.
    pub(crate) _guards: Vec<L::ReadGuard<'a, Shard<K, V>>>,
    /// One `hashbrown` iterator per shard, aligned with shard indices.
    pub(crate) shard_iters: Vec<ShardIter<'a, (K, V)>>,
    pub(crate) shard_index: usize,
    /// Entries remaining in shards visited so far (an exact lower bound).
    pub(crate) remaining: usize,
}

impl<'a, K, V, L> Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    pub(crate) fn new(custodian: &'a Custodian<K, V, L>) -> Self {
        Self {
            custodian,
            _guards: Vec::with_capacity(custodian.shard_count.0 as usize),
            shard_iters: Vec::with_capacity(custodian.shard_count.0 as usize),
            shard_index: 0,
            remaining: 0,
        }
    }
}

impl<'a, K, V, L> Iterator for Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Lazily acquire the read guard for the next shard on first visit.
            if self.shard_index == self.shard_iters.len() {
                if self.shard_index >= self.custodian.shard_count.0 as usize {
                    return None;
                }
                let guard = self
                    .custodian
                    .read_guard_at(ShardIndex(self.shard_index as u8));
                self.remaining += guard.len();
                // SAFETY: `hashbrown`'s `Iter` stores only raw pointers into
                // the shard's heap-allocated buckets plus a `PhantomData`
                // marker; the lifetime is not tracked at runtime. The read
                // guard keeps the shard data alive and immutable for `'a`, and
                // is stored alongside the iterators in this struct, so the
                // iterators can never outlive the data they reference.
                let iter: ShardIter<'a, (K, V)> = unsafe { std::mem::transmute(guard.iter()) };
                self._guards.insert(self.shard_index, guard);
                self.shard_iters.push(iter);
            }
            let shard = &mut self.shard_iters[self.shard_index];
            if let Some(entry) = shard.next() {
                self.remaining -= 1;
                return Some((&entry.0, &entry.1));
            }
            self.shard_index += 1;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.shard_index >= self.custodian.shard_count.0 as usize;
        (self.remaining, exact.then_some(self.remaining))
    }
}

impl<'a, K, V, L> IntoIterator for &'a TxMap<K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V, L> IntoIterator for &'a mut TxMap<K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over all the keys in a [`TxMap`].
///
/// Created by [`TxMap::keys`]. Acquires read guards lazily, one shard at a
/// time, holding them until the iterator is dropped.
pub struct Keys<'a, K, V, L>(pub(crate) Iter<'a, K, V, L>)
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a;

impl<'a, K, V, L> Iterator for Keys<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// An iterator over all the values in a [`TxMap`].
///
/// Created by [`TxMap::values`]. Acquires read guards lazily, one shard at a
/// time, holding them until the iterator is dropped.
pub struct Values<'a, K, V, L>(pub(crate) Iter<'a, K, V, L>)
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a;

impl<'a, K, V, L> Iterator for Values<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// An owning iterator over all key-value pairs in a [`TxMap`], removing
/// each entry as it is yielded.
///
/// Created by [`TxMap::drain`]. Write guards are acquired lazily, one shard
/// at a time, as iteration progresses and held until the iterator is
/// dropped. Dropping the iterator without fully consuming it removes all
/// remaining entries.
pub struct Drain<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    /// The shard custodian, used to acquire write guards lazily.
    pub(crate) custodian: &'a Custodian<K, V, L>,
    /// One `hashbrown` drain per visited shard, aligned with shard indices.
    ///
    /// Declared before `_guards` so it is dropped first: on drop each
    /// drain clears its table while the corresponding write lock is still
    /// held.
    pub(crate) shard_drains: Vec<ShardDrain<'a, (K, V)>>,
    /// Write guards keeping every visited shard locked (and alive) for `'a`.
    pub(crate) _guards: Vec<L::WriteGuard<'a, Shard<K, V>>>,
    pub(crate) shard_index: usize,
    /// Entries remaining in shards visited so far (an exact lower bound).
    pub(crate) remaining: usize,
}

impl<'a, K, V, L> Drain<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    pub(crate) fn new(custodian: &'a Custodian<K, V, L>) -> Self {
        Self {
            custodian,
            shard_drains: Vec::with_capacity(custodian.shard_count.0 as usize),
            _guards: Vec::with_capacity(custodian.shard_count.0 as usize),
            shard_index: 0,
            remaining: 0,
        }
    }
}

impl<'a, K, V, L> Iterator for Drain<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Lazily acquire the write guard and drain for the next shard on
            // first visit.
            if self.shard_index == self.shard_drains.len() {
                if self.shard_index >= self.custodian.shard_count.0 as usize {
                    return None;
                }
                let mut guard = self
                    .custodian
                    .write_guard_at(ShardIndex(self.shard_index as u8));
                self.remaining += guard.len();
                // SAFETY: `hashbrown`'s `Drain` stores only raw pointers into
                // the shard's heap-allocated buckets plus a `PhantomData`
                // marker; the lifetime is not tracked at runtime. The write
                // guard keeps the shard data alive and exclusively locked for
                // `'a`, and is stored alongside the drains in this struct
                // (and dropped after them), so the drains can never outlive
                // the data they reference.
                let drain: ShardDrain<'a, (K, V)> = unsafe { std::mem::transmute(guard.drain()) };
                self._guards.push(guard);
                self.shard_drains.push(drain);
            }
            let shard = &mut self.shard_drains[self.shard_index];
            if let Some(entry) = shard.next() {
                self.remaining -= 1;
                return Some(entry);
            }
            self.shard_index += 1;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.shard_index >= self.custodian.shard_count.0 as usize;
        (self.remaining, exact.then_some(self.remaining))
    }
}

impl<'a, K, V, L> Drop for Drain<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    fn drop(&mut self) {
        // Shards already visited are cleared when their `ShardDrain` fields
        // are dropped (fields drop after this method, drains before guards).
        // Shards not yet visited are cleared here so that dropping the
        // iterator removes every remaining entry. Only shards beyond the ones
        // already locked are touched; the visited shards' write guards are
        // still held and must not be re-acquired.
        let mut shard_index = self.shard_drains.len();
        while shard_index < self.custodian.shard_count.0 as usize {
            let mut guard = self.custodian.write_guard_at(ShardIndex(shard_index as u8));
            guard.clear();
            shard_index += 1;
        }
    }
}
