use crate::{
    indexed_keys::IndexedKeys, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct RemoveOp<K>
where
    K: Hash + Eq,
{
    indexed_keys: IndexedKeys<K>,
}

impl<K> RemoveOp<K>
where
    K: Hash + Eq,
{
    pub fn new<I>(shard_count: u8, keys: I) -> Self
    where
        I: IntoIterator<Item = K>,
    {
        let indexed_keys = ShardCount::indexes(shard_count, keys, |k| k);
        Self { indexed_keys }
    }
}

impl<K, V, P> OpTrait<K, V, P> for RemoveOp<K>
where
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, _: &P) {
        for indexed_key in &self.indexed_keys.indexed {
            indexed_key.remove_entry(mutex_guards);
        }
    }
}
