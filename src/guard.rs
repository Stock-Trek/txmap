use crate::{
    indexer::{IndexedData, Indexer},
    result::INCORRECT_GUARD_VALUES_LENGTH,
};
use std::hash::Hash;

pub(crate) struct Guard<K, V>
where
    K: Clone + Hash + Eq,
{
    pub guards_bitmask: u128,
    pub name: String,
    pub indexed_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub is_condition_met: Box<dyn Fn(&[Option<&V>]) -> bool>,
}

impl<K, V> Guard<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn new<const N: usize, C>(
        indexer: Indexer,
        name: String,
        keys: [K; N],
        condition: C,
    ) -> Self
    where
        C: Fn([Option<&V>; N]) -> bool + 'static,
    {
        let indexed_keys = indexer.indexes(keys, |k| k);
        let is_condition_met = Box::new(move |values: &[Option<&V>]| {
            let array: [Option<&V>; N] = values.try_into().expect(INCORRECT_GUARD_VALUES_LENGTH);
            (condition)(array)
        });
        Self {
            guards_bitmask: indexed_keys.bitmask,
            name,
            indexed_keys,
            is_condition_met,
        }
    }
}
