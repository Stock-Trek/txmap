use crate::indexer::{IndexedData, Indexer};
use std::hash::Hash;

#[allow(dead_code)]
pub(crate) struct SkeletonOperation<K, V, P>
where
    K: Hash + Eq,
{
    pub guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    pub indexed_context_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub operator: Box<dyn Fn(Option<&V>, &[Option<&V>], &P) -> Option<V>>,
}

impl<K, V, P> SkeletonOperation<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new<O>(indexer: &Indexer, key: K, operator: O) -> Self
    where
        O: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            indexed_context_keys: IndexedData {
                bitmask: 0,
                indexed: vec![],
            },
            operator: Box::new(move |value, _, params| (operator)(value, params)),
        }
    }
    pub fn new_with_context<const N: usize, O>(
        indexer: &Indexer,
        key: K,
        operator: O,
        context_keys: [K; N],
    ) -> Self
    where
        O: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_context_keys = indexer.indexes(context_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_context_keys.bitmask,
            key_index,
            key,
            indexed_context_keys,
            operator: Box::new(move |value, context_values, params| {
                let context_array: [Option<&V>; N] = context_values
                    .try_into()
                    .expect("Incorrect operation values length");
                (operator)(value, context_array, params)
            }),
        }
    }
}
