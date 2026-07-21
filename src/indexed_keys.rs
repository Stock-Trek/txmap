use crate::{indexed_key::IndexedKey, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct IndexedKeys<K>
where
    K: Hash + Eq,
{
    pub bitmask: BitMask,
    pub indexed: Vec<IndexedKey<K>>,
}

impl<K> IndexedKeys<K>
where
    K: Hash + Eq,
{
    pub fn values<'guards, V>(
        &self,
        mutex_guards: &'guards IntMap<u8, MutexGuard<HashTable<(K, V)>>>,
    ) -> Vec<Option<&'guards V>> {
        let mut values = Vec::with_capacity(self.indexed.len());
        for indexed_peek_key in &self.indexed {
            let peek_value = indexed_peek_key.value_ref(mutex_guards);
            values.push(peek_value);
        }
        values
    }
}
