use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Indexer {
    pub shard_count: u64,
    pub hasher_creator: fn() -> Box<dyn Hasher>,
}

pub(crate) struct IndexedData<T> {
    pub bitmask: u128,
    pub indexed: Vec<(u8, T)>,
}

impl Indexer {
    pub fn indexes<'k, KI, E, K>(&self, keys: KI, element_to_key: fn(&E) -> &K) -> IndexedData<E>
    where
        KI: IntoIterator<Item = E>,
        K: Hash + 'k,
    {
        let mut bitmask: u128 = 0;
        let iter = keys.into_iter();
        let mut indexed = Vec::with_capacity(iter.size_hint().0);
        for element in iter {
            let key = element_to_key(&element);
            let index = self.index(key);
            bitmask |= 1 << index;
            indexed.push((index, element));
        }
        IndexedData { bitmask, indexed }
    }
    pub fn index<K>(&self, key: &K) -> u8
    where
        K: Hash,
    {
        let mut hasher = (self.hasher_creator)();
        key.hash(&mut hasher);
        (hasher.finish() % self.shard_count) as u8
    }
}
