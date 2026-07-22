use crate::{
    indexed_key::IndexedKey, indexed_keys::IndexedKeys, new_types::BitMask, ops::op_trait::OpTrait,
    result::INCORRECT_PEEK_VALUES_LENGTH, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct UpdatePeekOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    indexed_peek_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    transform: Box<dyn Fn(&K, Option<&V>, &[Option<&V>], &P) -> Option<V>>,
}

impl<K, V, P> UpdatePeekOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<const N: usize, T>(
        shard_count: u8,
        key: K,
        peek_keys: [K; N],
        transform: T,
    ) -> Self
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            indexed_peek_keys: ShardCount::indexes(shard_count, peek_keys, |k| k),
            transform: Box::new(move |key, value, peek_values, params| {
                let peek_array: [Option<&V>; N] =
                    peek_values.try_into().expect(INCORRECT_PEEK_VALUES_LENGTH);
                (transform)(key, value, peek_array, params)
            }),
        }
    }
}

impl<K, V> UpdatePeekOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<const N: usize, T>(shard_count: u8, key: K, peek_keys: [K; N], transform: T) -> Self
    where
        T: Fn(&K, Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        Self::new_with_params(shard_count, key, peek_keys, move |k, v, pks, _| {
            transform(k, v, pks)
        })
    }
}

impl<K, V, P> OpTrait<K, V, P> for UpdatePeekOp<K, V, P>
where
    K: Clone + Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2 | self.indexed_peek_keys.bitmask
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P) {
        let peek_values = self.indexed_peek_keys.values(mutex_guards);
        let value_ref = self.indexed_key.value_ref(mutex_guards);
        let new_value = (self.transform)(&self.indexed_key.3, value_ref, &peek_values, params);
        match new_value {
            Some(v) => self.indexed_key.insert(mutex_guards, v),
            None => {
                self.indexed_key.remove_entry(mutex_guards);
                ()
            }
        };
    }
}
