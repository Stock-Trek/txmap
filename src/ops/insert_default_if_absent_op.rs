use crate::{
    indexed_key::IndexedKey, locks::lock_policy::LockPolicy, new_types::BitMask,
    ops::op_trait::OpTrait, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOp<K>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
}

impl<K> InsertDefaultIfAbsentOp<K>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: u8, key: K) -> Self {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
        }
    }
}

impl<L, K, V, P> OpTrait<L, K, V, P> for InsertDefaultIfAbsentOp<K>
where
    L: LockPolicy,
    K: Clone + Hash + Eq,
    V: Default,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>, _: &P) {
        self.indexed_key
            .insert_if_absent::<L, V>(mutex_guards, || V::default());
    }
}
