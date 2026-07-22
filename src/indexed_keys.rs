use crate::{indexed_key::IndexedKey, locks::lock_policy::LockPolicy, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;
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
    pub fn values<'guards, L, V>(
        &self,
        mutex_guards: &'guards IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Vec<Option<&'guards V>>
    where
        L: LockPolicy,
        K: 'guards,
    {
        let mut values = Vec::with_capacity(self.indexed.len());
        for indexed_peek_key in &self.indexed {
            let peek_value = indexed_peek_key.value_ref::<L, V>(mutex_guards);
            values.push(peek_value);
        }
        values
    }
}
