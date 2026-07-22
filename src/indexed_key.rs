use crate::{
    locks::lock_policy::LockPolicy,
    new_types::{BitMask, HashCode, ShardIndex},
    result::MISSING_MUTEX_GUARD_ERROR,
    shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct IndexedKey<K>(pub HashCode, pub ShardIndex, pub BitMask, pub K)
where
    K: Hash + Eq;

impl<K> IndexedKey<K>
where
    K: Hash + Eq,
{
    pub fn insert<L, V>(
        &self,
        mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        value: V,
    ) where
        L: LockPolicy,
        K: Clone,
    {
        let mutex_guard = mutex_guards
            .get_mut(self.1.0)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard
            .entry(
                self.0.0,
                |entry| entry.0 == self.3,
                |entry| ShardCount::hash(&entry.0).0,
            )
            .insert((self.3.clone(), value));
    }
    pub fn insert_if_absent<L, V>(
        &self,
        mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        value_gen: impl FnOnce() -> V,
    ) where
        L: LockPolicy,
        K: Clone,
    {
        let mutex_guard = mutex_guards
            .get_mut(self.1.0)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard
            .entry(
                self.0.0,
                |entry| entry.0 == self.3,
                |entry| ShardCount::hash(&entry.0).0,
            )
            .or_insert_with(|| (self.3.clone(), value_gen()));
    }
    pub fn insert_with_duplicate_key<L, V>(
        &self,
        mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        duplicate_key: K,
        value: V,
    ) where
        L: LockPolicy,
    {
        assert!(duplicate_key == self.3);
        let mutex_guard = mutex_guards
            .get_mut(self.1.0)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard
            .entry(
                self.0.0,
                |entry| entry.0 == self.3,
                |entry| ShardCount::hash(&entry.0).0,
            )
            .insert((duplicate_key, value));
    }
    pub fn remove_entry<L, V>(
        &self,
        mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Option<(K, V)>
    where
        L: LockPolicy,
    {
        let mutex_guard = mutex_guards
            .get_mut(self.1.0)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard
            .find_entry(self.0.0, |entry| entry.0 == self.3)
            .ok()
            .map(|entry| entry.remove().0)
    }
    pub fn value_ref<'guards, L, V>(
        &self,
        mutex_guards: &'guards IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Option<&'guards V>
    where
        L: LockPolicy,
        K: 'guards,
    {
        let mutex_guard = mutex_guards.get(self.1.0).expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard
            .find(self.0.0, |entry| entry.0 == self.3)
            .map(|(_key, value)| value)
    }
}
