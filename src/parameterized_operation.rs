use crate::{
    indexer::{IndexedData, Indexer},
    result::INCORRECT_PEEK_VALUES_LENGTH,
};
use std::hash::Hash;

pub(crate) struct ParameterizedOperation<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    pub key_index: u8,
    pub key: K,
    pub indexed_peek_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub operator: Box<dyn Fn(Option<&V>, &[Option<&V>], &P) -> Option<V>>,
}

impl<K, V, P> ParameterizedOperation<K, V, P>
where
    K: Clone + Hash + Eq,
{
    pub fn new<F>(indexer: &Indexer, key: K, operator: F) -> Self
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            indexed_peek_keys: IndexedData {
                bitmask: 0,
                indexed: vec![],
            },
            operator: Box::new(move |value, _, params| (operator)(value, params)),
        }
    }
    pub fn new_with_context<const N: usize, F>(
        indexer: &Indexer,
        key: K,
        operator: F,
        peek_keys: [K; N],
    ) -> Self
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let key_index = indexer.index(&key);
        let indexed_peek_keys = indexer.indexes(peek_keys, |k| k);
        Self {
            guards_bitmask: (1 << key_index) | indexed_peek_keys.bitmask,
            key_index,
            key,
            indexed_peek_keys,
            operator: Box::new(move |value, peek_values, params| {
                let peek_array: [Option<&V>; N] =
                    peek_values.try_into().expect(INCORRECT_PEEK_VALUES_LENGTH);
                (operator)(value, peek_array, params)
            }),
        }
    }
}
