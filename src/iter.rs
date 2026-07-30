use crate::{lock_policies::lock_policy::LockPolicy, shard::Shard, tx_map::TxMap};
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
    pub(crate) _guards: IntMap<u8, L::ReadGuard<'a, Shard<K, V>>>,
    pub(crate) shard_index: u8,
    pub(crate) bucket_index: usize,
    pub(crate) shard_count: u8,
    pub(crate) remaining: usize,
}

impl<'a, K, V, L> Iterator for Iter<'a, K, V, L>
where
    K: Clone + Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while (self.shard_index as usize) < self.shard_count as usize {
            if let Some(guard) = self._guards.get(self.shard_index) {
                let num_buckets = guard.num_buckets();
                while self.bucket_index < num_buckets {
                    if let Some(entry) = guard.get_bucket(self.bucket_index) {
                        self.bucket_index += 1;
                        self.remaining -= 1;
                        // SAFETY: The guard is stored in `self._guards` and has
                        // lifetime `'a`. The entry reference obtained from
                        // `get_bucket` borrows the guard, but we extend it to `'a`
                        // because the guard keeps the shard data locked and alive
                        // for the entire lifetime of this iterator.
                        let entry_ref = unsafe { &*(entry as *const (K, V)) };
                        return Some((&entry_ref.0, &entry_ref.1));
                    }
                    self.bucket_index += 1;
                }
            }
            self.shard_index += 1;
            self.bucket_index = 0;
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
