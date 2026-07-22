use crate::{
    finishers::finisher_trait::FinisherTrait, indexed_key::IndexedKey,
    locks::lock_policy::LockPolicy, new_types::BitMask, shard::Shard, shard_count::ShardCount,
};
use intmap::IntMap;
use std::hash::Hash;

pub struct ValueFinisher<K, V, R>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>) -> Option<R>>,
}

impl<K, V, R> ValueFinisher<K, V, R>
where
    K: Hash + Eq,
{
    pub fn new<T>(shard_count: u8, key: K, transform: T) -> Self
    where
        T: Fn(&K, &V) -> R + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            transform: Box::new(move |key, value_opt| value_opt.map(|value| transform(key, value))),
        }
    }
}

impl<K, V, R> FinisherTrait<K, V> for ValueFinisher<K, V, R>
where
    K: Hash + Eq,
{
    type Output = Option<R>;

    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn to_result<L>(&self, mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>) -> Option<R>
    where
        L: LockPolicy,
    {
        let value_ref = self.indexed_key.value_ref::<L, V>(mutex_guards);
        (self.transform)(&self.indexed_key.3, value_ref)
    }
}
