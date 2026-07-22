use crate::{
    indexed_keys::IndexedKeys, locks::lock_policy::LockPolicy, new_types::BitMask,
    ops::op_trait::OpTrait, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct RemoveWhereOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    condition: Box<dyn Fn(&K, &V, &P) -> bool>,
}

impl<K, V, P> RemoveWhereOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<I, C>(shard_count: u8, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V, &P) -> bool + 'static,
    {
        let indexed_keys = ShardCount::indexes(shard_count, keys, |k| k);
        Self {
            indexed_keys,
            condition: Box::new(condition),
        }
    }
}

impl<K, V> RemoveWhereOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<I, C>(shard_count: u8, keys: I, condition: C) -> Self
    where
        I: IntoIterator<Item = K>,
        C: Fn(&K, &V) -> bool + 'static,
    {
        Self::new_with_params(shard_count, keys, move |k, v, _| condition(k, v))
    }
}

impl<L, K, V, P> OpTrait<L, K, V, P> for RemoveWhereOp<K, V, P>
where
    L: LockPolicy,
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>, params: &P) {
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref::<L, V>(mutex_guards);
            if let Some(v) = value_ref
                && (self.condition)(&indexed_key.3, v, params)
            {
                indexed_key.remove_entry::<L, V>(mutex_guards);
            }
        }
    }
}
