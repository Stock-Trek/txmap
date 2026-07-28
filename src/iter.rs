use crate::{lock_policies::lock_policy::LockPolicy, shard::Shard, tx_map::TxMap};
use intmap::IntMap;
use std::hash::Hash;

/// An iterator over the key-value pairs of a [`TxMap`].
///
/// This struct is created by the [`iter`](TxMap::iter) method on [`TxMap`] or by calling
/// [`IntoIterator`] on a reference to a [`TxMap`].
pub struct Iter<'a, K, V, L>
where
    K: Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    /// Guards keep the shard data alive and locked for the lifetime of the iterator.
    pub(crate) _guards: IntMap<u8, L::ReadGuard<'a, Shard<K, V>>>,
    /// Raw pointers to all key-value pairs across all shards.
    /// The data lives as long as `_guards` (and thus as long as this struct).
    pub(crate) entries: Vec<*const (K, V)>,
    pub(crate) index: usize,
}

impl<'a, K, V, L> Iterator for Iter<'a, K, V, L>
where
    K: Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let entry = self.entries[self.index];
            self.index += 1;
            // SAFETY: The entry pointer points into a `HashTable` behind a
            // `ReadGuard` stored in `self._guards`. The guards keep the data
            // alive for `'a`, and this iterator cannot outlive the struct.
            let entry_ref = unsafe { &*entry };
            Some((&entry_ref.0, &entry_ref.1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.entries.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a, K, V, L> IntoIterator for &'a TxMap<K, V, L>
where
    K: Hash + Eq + 'a,
    V: 'a,
    L: LockPolicy + 'a,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
