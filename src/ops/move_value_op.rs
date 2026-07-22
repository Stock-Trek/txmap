use crate::{
    indexed_key::IndexedKey, locks::lock_policy::LockPolicy, new_types::BitMask,
    ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct MoveValueOp<K>
where
    K: Hash + Eq,
{
    indexed_key_from: IndexedKey<K>,
    indexed_key_to: IndexedKey<K>,
}

impl<K> MoveValueOp<K>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: u8, from: K, to: K) -> Self {
        Self {
            indexed_key_from: ShardCount::indexed_key(shard_count, from),
            indexed_key_to: ShardCount::indexed_key(shard_count, to),
        }
    }
}

impl<L, K, V, P> OpTrait<L, K, V, P> for MoveValueOp<K>
where
    L: LockPolicy,
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key_from.2 | self.indexed_key_to.2
    }
    fn apply<'guards>(
        &self,
        mutex_guards: &'guards mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        _: &P,
    ) {
        let removed = self.indexed_key_from.remove_entry::<L, V>(mutex_guards);
        if let Some(entry) = removed {
            self.indexed_key_to.insert::<L, V>(mutex_guards, entry.1);
        } else {
            self.indexed_key_to.remove_entry::<L, V>(mutex_guards);
        }
    }
}
