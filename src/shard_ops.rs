use hashbrown::hash_table::Entry;

use crate::{indexer::Indexer, key::TxKey, shard::Shard};
use std::hash::{BuildHasher, Hash};

pub(crate) struct ShardOps;

impl ShardOps {
    #[inline]
    pub fn value_ref<'op, K, V>(shard: &'op Shard<K, V>, key: &TxKey<K>) -> Option<&'op V>
    where
        K: Clone + Hash + Eq,
    {
        shard
            .find(key.hash_code.0, |entry| entry.0 == key.key)
            .map(|(_key, value)| value)
    }

    #[inline]
    pub fn get_or_insert<'op, K, V, S>(
        shard: &'op mut Shard<K, V>,
        key: &TxKey<K>,
        value: V,
        indexer: &Indexer<S>,
    ) -> &'op V
    where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => &occupied.into_mut().1,
            Entry::Vacant(vacant) => &vacant.insert((key.key.clone(), value)).into_mut().1,
        }
    }

    #[inline]
    pub fn get_or_insert_with<'op, K, V, S>(
        shard: &'op mut Shard<K, V>,
        key: &TxKey<K>,
        value_gen: impl FnOnce(&K) -> V,
        indexer: &Indexer<S>,
    ) -> &'op V
    where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => &occupied.into_mut().1,
            Entry::Vacant(vacant) => {
                let value = value_gen(&key.key);
                &vacant.insert((key.key.clone(), value)).into_mut().1
            }
        }
    }

    #[inline]
    pub fn insert<K, V, S>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        value: V,
        indexer: &Indexer<S>,
    ) -> Option<V>
    where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => {
                let mut old_value = None;
                occupied.replace_entry_with(|entry| {
                    old_value = Some(entry.1);
                    Some((entry.0, value))
                });
                old_value
            }
            Entry::Vacant(vacant) => {
                vacant.insert((key.key.clone(), value));
                None
            }
        }
    }

    #[inline]
    pub fn insert_if_absent<K, V, S>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        value_gen: impl FnOnce() -> V,
        indexer: &Indexer<S>,
    ) -> bool
    where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(_) => false,
            Entry::Vacant(vacant) => {
                vacant.insert((key.key.clone(), value_gen()));
                true
            }
        }
    }

    #[inline]
    pub fn insert_with_duplicate_key<K, V, S>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        duplicate_key: K,
        value: V,
        indexer: &Indexer<S>,
    ) where
        K: Hash + Eq,
        S: BuildHasher,
    {
        shard
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| indexer.hash(&entry.0).0,
            )
            .insert((duplicate_key, value));
    }

    #[inline]
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

    #[inline]
    pub fn remove_entry<K, V>(shard: &mut Shard<K, V>, key: &TxKey<K>) -> Option<(K, V)>
    where
        K: Hash + Eq,
    {
        shard
            .find_entry(key.hash_code.0, |entry| entry.0 == key.key)
            .ok()
            .map(|entry| entry.remove().0)
    }

    #[inline]
    pub fn remove_if<K, V, S>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        condition: impl FnOnce(&K, &V) -> bool,
        indexer: &Indexer<S>,
    ) -> Option<V>
    where
        K: Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => {
                let (found_key, found_value) = occupied.get();
                if condition(found_key, found_value) {
                    Some(occupied.remove().0.1)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => None,
        }
    }

    #[inline]
    pub fn update<K, V, S>(
        shard: &mut Shard<K, V>,
        key: &TxKey<K>,
        transform: impl FnOnce(&K, Option<&V>) -> Option<V>,
        indexer: &Indexer<S>,
    ) where
        K: Clone + Hash + Eq,
        S: BuildHasher,
    {
        let entry = shard.entry(
            key.hash_code.0,
            |entry| entry.0 == key.key,
            |entry| indexer.hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => {
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
            Entry::Vacant(vacant) => {
                if let Some(new_value) = transform(&key.key, None) {
                    vacant.insert((key.key.clone(), new_value));
                }
            }
        }
    }
}
