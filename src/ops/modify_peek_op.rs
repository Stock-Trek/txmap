use crate::{
    indexed_key::IndexedKey, indexed_keys::IndexedKeys, new_types::BitMask, ops::op_trait::OpTrait,
    result::INCORRECT_PEEK_VALUES_LENGTH, shard_count::ShardCount,
};
use hashbrown::HashTable;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub(crate) struct ModifyPeekOp<K, V, P = ()>
where
    K: Hash + Eq,
{
    indexed_key: IndexedKey<K>,
    indexed_peek_keys: IndexedKeys<K>,
    #[allow(clippy::type_complexity)]
    mutate: Box<dyn Fn(&K, &mut V, &[Option<&V>], &P)>,
}

impl<K, V, P> ModifyPeekOp<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new_with_params<const N: usize, M>(
        shard_count: u8,
        key: K,
        peek_keys: [K; N],
        mutate: M,
    ) -> Self
    where
        M: Fn(&K, &mut V, [Option<&V>; N], &P) + 'static,
    {
        Self {
            indexed_key: ShardCount::indexed_key(shard_count, key),
            indexed_peek_keys: ShardCount::indexes(shard_count, peek_keys, |k| k),
            mutate: Box::new(move |key, value, peek_values, params| {
                let peek_array: [Option<&V>; N] =
                    peek_values.try_into().expect(INCORRECT_PEEK_VALUES_LENGTH);
                (mutate)(key, value, peek_array, params)
            }),
        }
    }
}

impl<K, V> ModifyPeekOp<K, V, ()>
where
    K: Hash + Eq,
{
    pub fn new<const N: usize, M>(shard_count: u8, key: K, peek_keys: [K; N], mutate: M) -> Self
    where
        M: Fn(&K, &mut V, [Option<&V>; N]) + 'static,
    {
        Self::new_with_params(shard_count, key, peek_keys, move |k, v, pks, _| {
            mutate(k, v, pks)
        })
    }
}

impl<K, V, P> OpTrait<K, V, P> for ModifyPeekOp<K, V, P>
where
    K: Hash + Eq,
{
    fn guards_bitmask(&self) -> BitMask {
        self.indexed_key.2
    }
    fn apply(&self, mutex_guards: &mut IntMap<u8, MutexGuard<HashTable<(K, V)>>>, params: &P) {
        // It's not possible to read peeked values while modifying the key value in place
        // Therefore we:
        // 1 .Remove the value
        // 2. Get the read-only peeked values
        // 3. Allow the user to modify the removed value
        // 4. Re-insert the modified value
        if let Some((duplicate_key, mut value)) = self.indexed_key.remove_entry(mutex_guards) {
            let peek_values = self.indexed_peek_keys.values(mutex_guards);
            (self.mutate)(
                &self.indexed_key.3,
                &mut value,
                peek_values.as_slice(),
                params,
            );
            self.indexed_key
                .insert_with_duplicate_key(mutex_guards, duplicate_key, value);
        }
    }
}
