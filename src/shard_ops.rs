use crate::{indexer::Indexer, key::TxKey, shard::Shard};
use std::hash::Hash;

pub(crate) struct ShardOps;

impl ShardOps {
    pub fn value_ref<'op, K, V>(shard: &'op Shard<K, V>, key: &TxKey<K>) -> Option<&'op V>
    where
        K: Clone + Hash + Eq,
    {
        shard
            .find(key.hash_code.0, |entry| entry.0 == key.key)
            .map(|(_key, value)| value)
    }
    pub fn insert<K, V>(shard: &mut Shard<K, V>, key: &TxKey<K>, value: V) -> Option<V>
    where
        K: Clone + Hash + Eq,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            hashbrown::hash_table::Entry::Occupied(occupied) => {
                let ((old_key, old_value), vacant) = occupied.remove();
                vacant.insert((old_key, value));
                Some(old_value)
            }
            hashbrown::hash_table::Entry::Vacant(vacant) => {
                vacant.insert((key.key.clone(), value));
                None
            }
        }
    }
    pub fn insert_if_absent<K, V>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        value_gen: impl FnOnce() -> V,
    ) -> bool
    where
        K: Clone + Hash + Eq,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            hashbrown::hash_table::Entry::Occupied(_) => false,
            hashbrown::hash_table::Entry::Vacant(vacant) => {
                vacant.insert((key.key.clone(), value_gen()));
                true
            }
        }
    }
    pub fn insert_with_duplicate_key<K, V>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        duplicate_key: K,
        value: V,
    ) where
        K: Hash + Eq,
    {
        shard
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| Indexer::hash(&entry.0).0,
            )
            .insert((duplicate_key, value));
    }
    pub fn modify<K, V>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        mutate: impl FnOnce(&K, &mut V),
    ) -> bool
    where
        K: Hash + Eq,
    {
        if let Some(mut_entry) = shard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            mutate(&mut_entry.0, &mut mut_entry.1);
            true
        } else {
            false
        }
    }
    pub fn remove_entry<K, V>(shard: &mut Shard<K, V>, key: &TxKey<K>) -> Option<(K, V)>
    where
        K: Hash + Eq,
    {
        shard
            .find_entry(key.hash_code.0, |entry| entry.0 == key.key)
            .ok()
            .map(|entry| entry.remove().0)
    }
    pub fn remove_if<K, V>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        condition: impl FnOnce(&K, &V) -> bool,
    ) -> Option<V>
    where
        K: Hash + Eq,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            hashbrown::hash_table::Entry::Occupied(occupied) => {
                let (found_key, found_value) = occupied.get();
                if condition(found_key, found_value) {
                    Some(occupied.remove().0.1)
                } else {
                    None
                }
            }
            hashbrown::hash_table::Entry::Vacant(_) => None,
        }
    }
    pub fn update<K, V>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        transform: impl FnOnce(&K, Option<&V>) -> Option<V>,
    ) where
        K: Clone + Hash + Eq,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            hashbrown::hash_table::Entry::Occupied(occupied) => {
                let (found_key, found_value) = occupied.get();
                match transform(found_key, Some(found_value)) {
                    Some(new_value) => {
                        occupied.replace_entry_with(|entry| Some((entry.0, new_value)));
                    }
                    None => {
                        occupied.remove();
                    }
                }
            }
            hashbrown::hash_table::Entry::Vacant(vacant) => {
                if let Some(new_value) = transform(&key.key, None) {
                    vacant.insert((key.key.clone(), new_value));
                }
            }
        }
    }
}
