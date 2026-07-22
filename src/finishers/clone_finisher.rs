use crate::{
    finishers::finisher_trait::FinisherTrait, indexed_key::IndexedKey, new_types::BitMask,
    shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::{hash::Hash, marker::PhantomData};

pub struct CloneFinisher<K, V>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> CloneFinisher<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    pub fn new(shard_count: u8, key: K) -> Self {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> FinisherTrait<K, V> for CloneFinisher<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Output = Option<V>;

    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn to_result(&self, mutex_guards: &IntMap<u8, MutexGuard<HashTable<(K, V)>>>) -> Option<V> {
        let value_ref = self.indexed_key.value_ref(mutex_guards);
        value_ref.cloned()
    }
}
