use crate::{
    finishers::finisher_trait::FinisherTrait, indexed_keys::IndexedKeys,
    locks::lock_policy::LockPolicy, new_types::BitMask, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use std::hash::Hash;

pub struct ValuesFinisher<K, V, R>
where
    K: Hash + Eq,
{
    indexed_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>) -> Option<R>>,
}

impl<K, V, R> ValuesFinisher<K, V, R>
where
    K: Hash + Eq,
{
    pub fn new<I, T>(shard_count: u8, keys: I, transform: T) -> Self
    where
        I: IntoIterator<Item = K>,
        T: Fn(&K, &V) -> R + 'static,
    {
        Self {
            indexed_keys: ShardCount::indexes(shard_count, keys, |k| k),
            transform: Box::new(move |key, value_opt| value_opt.map(|value| transform(key, value))),
        }
    }
}

impl<K, V, R> FinisherTrait<K, V> for ValuesFinisher<K, V, R>
where
    K: Hash + Eq,
{
    type Output = Vec<Option<R>>;

    fn guards_bitmask(&self) -> BitMask {
        self.indexed_keys.bitmask
    }
    fn to_result<'guards, L>(
        &self,
        mutex_guards: &'guards IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Vec<Option<R>>
    where
        L: LockPolicy,
    {
        let mut result = Vec::with_capacity(self.indexed_keys.indexed.len());
        for indexed_key in &self.indexed_keys.indexed {
            let value_ref = indexed_key.value_ref::<L, V>(mutex_guards);
            let result_value = (self.transform)(&indexed_key.3, value_ref);
            result.push(result_value);
        }
        result
    }
}
