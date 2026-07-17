use crate::indexer::{IndexedData, Indexer};
use std::hash::Hash;

#[allow(dead_code)]
pub(crate) struct SkeletonPrerequisite<K, V, P>
where
    K: Hash + Eq,
{
    pub guards_bitmask: u128,
    pub name: String,
    pub indexed_keys: IndexedData<K>,
    #[allow(clippy::type_complexity)]
    pub is_satisfied: Box<dyn Fn(&[Option<&V>], &P) -> bool>,
}

impl<K, V, P> SkeletonPrerequisite<K, V, P>
where
    K: Hash + Eq,
{
    pub fn new<const N: usize, F>(
        indexer: Indexer,
        name: String,
        keys: [K; N],
        prerequisite: F,
    ) -> Self
    where
        F: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let indexed_keys = indexer.indexes(keys, |k| k);
        let is_satisfied = Box::new(move |values: &[Option<&V>], params: &P| {
            let array: [Option<&V>; N] = values
                .try_into()
                .expect("Incorrect prerequisite values length");
            (prerequisite)(array, params)
        });
        Self {
            guards_bitmask: indexed_keys.bitmask,
            name,
            indexed_keys,
            is_satisfied,
        }
    }
}
