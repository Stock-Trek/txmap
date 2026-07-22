use crate::{
    finishers::finisher_trait::FinisherTrait, indexed_key::IndexedKey,
    locks::lock_policy::LockPolicy, new_types::BitMask, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::{hash::Hash, marker::PhantomData};

pub struct CopyFinisher<K, V>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    _phantom: PhantomData<V>,
}

impl<K, V> CopyFinisher<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    pub fn new(shard_count: u8, key: K) -> Self {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            _phantom: PhantomData,
        }
    }
}

impl<K, V> FinisherTrait<K, V> for CopyFinisher<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    type Output = Option<V>;

    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn to_result<L>(&self, mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>) -> Option<V>
    where
        L: LockPolicy,
    {
        let value_ref = self.indexed_key.value_ref::<L, V>(mutex_guards);
        value_ref.copied()
    }
}
