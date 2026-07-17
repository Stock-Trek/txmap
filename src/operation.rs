use crate::indexer::{IndexedData, Indexer};
use std::hash::Hash;

pub(crate) struct Operation<K, V>
where
    K: Hash + Eq,
{
    pub guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    pub indexed_context_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub operator: Box<dyn Fn(Option<&V>, &[Option<&V>]) -> Option<V>>,
}

impl<K, V> Operation<K, V>
where
    K: Hash + Eq,
{
    pub fn new<F>(indexer: &Indexer, key: K, operator: F) -> Self
    where
        F: Fn(Option<&V>) -> Option<V> + 'static,
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
            operator: Box::new(move |value, _| (operator)(value)),
        }
    }
    pub fn new_with_context<const N: usize, F>(
        indexer: &Indexer,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> Self
    where
        F: Fn(Option<&V>, [Option<&V>; N]) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_context_keys = indexer.indexes(context_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_context_keys.bitmask,
            key_index,
            key,
            indexed_context_keys,
            operator: Box::new(move |value, context_values| {
                let context_array: [Option<&V>; N] = context_values
                    .try_into()
                    .expect("Incorrect operation values length");
                (operator)(value, context_array)
            }),
        }
    }
}
