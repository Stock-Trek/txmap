use crate::{
    indexed_key::IndexedKey, new_types::BitMask, ops::op_trait::OpTrait, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct SwapValueOp<K>
where
    K: Hash + Eq,
{
    indexed_key_a: IndexedKey<K>,
    indexed_key_b: IndexedKey<K>,
}

impl<K> SwapValueOp<K>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: u8, a: K, b: K) -> Self {
        Self {
            indexed_key_a: ShardCount::indexed_key(shard_count, a),
            indexed_key_b: ShardCount::indexed_key(shard_count, b),
        }
    }
}

impl<K, V, P> OpTrait<K, V, P> for SwapValueOp<K>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key_a.2 | self.indexed_key_b.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, _: &P) {
        let a = self.indexed_key_a.remove_entry(mutex_guards);
        let b = self.indexed_key_b.remove_entry(mutex_guards);
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    self.indexed_key_a
                        .insert_with_duplicate_key(mutex_guards, a_key, b_value);
                    self.indexed_key_b
                        .insert_with_duplicate_key(mutex_guards, b_key, a_value);
                }
                None => {
                    self.indexed_key_b.insert(mutex_guards, a_value);
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    self.indexed_key_a.insert(mutex_guards, b_value);
                }
            }
        }
    }
}
