use crate::{
    indexer::Indexer, key::TxKey, lock_policies::lock_policy::LockPolicy, new_types::BitMask,
    result::MISSING_LOCK_GUARD_ERROR, shard::Shard,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct LockGuards<'ex, K, V, L>
where
    K: 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub read: IntMap<u8, L::ReadGuard<'ex, Shard<K, V>>>,
    pub write: IntMap<u8, L::WriteGuard<'ex, Shard<K, V>>>,
    pub write_bitmask: BitMask,
}

impl<'ex, K, V, L> LockGuards<'ex, K, V, L>
where
    K: 'ex,
    V: 'ex,
    L: LockPolicy + 'ex,
{
    pub fn insert(&mut self, key: &TxKey<K>, value: V) -> Option<V>
    where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let shard = self
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
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
    pub fn insert_if_absent(&mut self, key: &TxKey<K>, value_gen: impl FnOnce() -> V) -> bool
    where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let shard = self
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
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
    pub fn insert_with_duplicate_key(&mut self, key: &TxKey<K>, duplicate_key: K, value: V)
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .entry(
                key.hash_code.0,
                |entry| entry.0 == key.key,
                |entry| Indexer::hash(&entry.0).0,
            )
            .insert((duplicate_key, value));
    }
    pub fn modify(&mut self, key: &TxKey<K>, mutate: impl FnOnce(&K, &mut V)) -> bool
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = self
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        if let Some(mut_entry) = write_guard.find_mut(key.hash_code.0, |entry| entry.0 == key.key) {
            mutate(&mut_entry.0, &mut mut_entry.1);
            true
        } else {
            false
        }
    }
    pub fn remove_entry(&mut self, key: &TxKey<K>) -> Option<(K, V)>
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        self.write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .find_entry(key.hash_code.0, |entry| entry.0 == key.key)
            .ok()
            .map(|entry| entry.remove().0)
    }
    pub fn remove_if(&mut self, key: &TxKey<K>, condition: impl FnOnce(&K, &V) -> bool) -> Option<V>
    where
        K: Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = self
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        let entry = write_guard.entry(
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
    pub fn update(&mut self, key: &TxKey<K>, transform: impl FnOnce(&K, Option<&V>) -> Option<V>)
    where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let write_guard = self
            .write
            .get_mut(key.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR);
        let entry = write_guard.entry(
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
