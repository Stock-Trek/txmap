use crate::{
    indexed_key::IndexedKey, locks::lock_policy::LockPolicy, new_types::BitMask,
    ops::op_trait::OpTrait, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    #[allow(clippy::type_complexity)]
    value_generator: Box<dyn Fn(&K, &P) -> V>,
}

impl<K, V, P> InsertWithIfAbsentOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<G>(shard_count: u8, key: K, value_generator: G) -> Self
    where
        G: Fn(&K, &P) -> V + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            value_generator: Box::new(value_generator),
        }
    }
}

impl<K, V> InsertWithIfAbsentOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<G>(shard_count: u8, key: K, value_generator: G) -> Self
    where
        G: Fn(&K) -> V + 'static,
    {
        Self::new_with_params(shard_count, key, move |k, _| value_generator(k))
    }
}

impl<L, K, V, P> OpTrait<L, K, V, P> for InsertWithIfAbsentOp<K, V, P>
where
    L: LockPolicy,
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>, params: &P) {
        self.indexed_key.insert_if_absent::<L, V>(mutex_guards, || {
            (self.value_generator)(&self.indexed_key.3, params)
        });
    }
}
