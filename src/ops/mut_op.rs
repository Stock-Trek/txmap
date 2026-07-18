use crate::indexer::Indexer;
use std::hash::Hash;

pub(crate) struct MutOp<K, V>
where
    K: Clone + Hash + Eq,
{
    guards_bitmask: u128,
    key_index: u8,
    key: K,
    mutator: Box<dyn Fn(&mut V)>,
    value_generator: Option<Box<dyn Fn() -> V>>,
}

impl<K, V> MutOp<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<M>(indexer: &Indexer, key: K, mutate: M) -> Self
    where
        M: Fn(&mut V) + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutator: Box::new(mutate),
            value_generator: None,
        }
    }
    pub fn new_or_insert_with<M, G>(
        indexer: &Indexer,
        key: K,
        mutate: M,
        value_generator: G,
    ) -> Self
    where
        M: Fn(&mut V) + 'static,
        G: Fn() -> V + 'static,
    {
        let key_index = indexer.index(&key);
        Self {
            guards_bitmask: 1 << key_index,
            key_index,
            key,
            mutator: Box::new(mutate),
            value_generator: Some(Box::new(value_generator)),
        }
    }
}
