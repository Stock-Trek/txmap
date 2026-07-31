use crate::{
    lock_policies::lock_policy::LockPolicy, result::MISSING_LOCK_GUARD_ERROR, shard::Shard,
    tx_map::TxMap,
};
use hashbrown::hash_table::Iter as ShardIter;
use intmap::IntMap;
use std::hash::Hash;

/// An iterator over all key-value pairs in a [`TxMap`].
///
/// Holds read guards on all shards for the duration of iteration.
pub struct Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    /// Read guards keeping every shard locked (and alive) for `'a`.
    pub(crate) _guards: IntMap<u8, L::ReadGuard<'a, Shard<K, V>>>,
    /// One `hashbrown` iterator per shard, aligned with shard indices.
    pub(crate) shard_iters: Vec<ShardIter<'a, (K, V)>>,
    pub(crate) shard_index: usize,
    pub(crate) remaining: usize,
}

impl<'a, K, V, L> Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    pub(crate) fn new(
        guards: IntMap<u8, L::ReadGuard<'a, Shard<K, V>>>,
        shard_count: u8,
        remaining: usize,
    ) -> Self {
        let mut shard_iters = Vec::with_capacity(shard_count as usize);
        for shard_index in 0..shard_count {
            let guard = guards
                .get(shard_index)
                .expect(MISSING_LOCK_GUARD_ERROR);
            // SAFETY: `hashbrown`'s `Iter` stores only raw pointers into the
            // shard's heap-allocated buckets plus a `PhantomData` marker; the
            // lifetime is not tracked at runtime. The read guard keeps the
            // shard data alive and immutable for `'a`, and is stored
            // alongside the iterators in this struct, so the iterators can
            // never outlive the data they reference.
            let iter: ShardIter<'a, (K, V)> = unsafe { std::mem::transmute(guard.iter()) };
            shard_iters.push(iter);
        }
        Self {
            _guards: guards,
            shard_iters,
            shard_index: 0,
            remaining,
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
        while self.shard_index < self.shard_iters.len() {
            let shard = &mut self.shard_iters[self.shard_index];
            if let Some(entry) = shard.next() {
                self.remaining -= 1;
                return Some((&entry.0, &entry.1));
            }
            self.shard_index += 1;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
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
