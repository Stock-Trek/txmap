use crate::{
    finishers::finisher_trait::FinisherTrait, indexed_keys::IndexedKeys,
    locks::lock_policy::LockPolicy, new_types::BitMask, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::{hash::Hash, marker::PhantomData};

pub struct CloneAllFinisher<K, V>
where
    K: Hash + Eq,
{
    indexed_keys: IndexedKeys<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> CloneAllFinisher<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    pub fn new(shard_count: u8, keys: impl IntoIterator<Item = K>) -> Self {
        Self {
            indexed_keys: ShardCount::indexes(shard_count, keys, |k| k),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> FinisherTrait<K, V> for CloneAllFinisher<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    type Output = Vec<Option<V>>;

    fn guards_bitmask(&self) -> BitMask {
        self.indexed_keys.bitmask
    }
    fn to_result<L>(
        &self,
        mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>,
    ) -> Vec<Option<V>>
    where
        L: LockPolicy,
    {
        let mut result = Vec::with_capacity(self.indexed_keys.indexed.len());
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref::<L, V>(mutex_guards);
            result.push(value_ref.cloned());
        }
        result
    }
}
