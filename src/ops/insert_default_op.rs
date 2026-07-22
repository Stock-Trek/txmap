use crate::{
    indexed_key::IndexedKey, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct InsertDefaultOp<K>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
}

impl<K> InsertDefaultOp<K>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: u8, key: K) -> Self {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
        }
    }
}

impl<K, V, P> OpTrait<K, V, P> for InsertDefaultOp<K>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, _: &P) {
        self.indexed_key.insert(mutex_guards, V::default());
    }
}
