use crate::{builders::stem_builder::TxStemBuilder, custodian::Custodian, shard_count::ShardCount};
use hashbrown::hash_table::Entry;
use std::hash::Hash;

pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    shard_count: u8,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    #[must_use]
    pub fn new(shard_count: ShardCount) -> Self {
        let shard_count = u8::from(shard_count);
        Self {
            shard_count,
            custodian: Custodian::new(shard_count),
        }
    }
    pub fn clear(&self) {
        for mut mutex_guard in self.custodian.all_guards() {
            mutex_guard.1.clear();
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        for mutex_guard in self.custodian.all_guards() {
            total_length += mutex_guard.1.len();
        }
        total_length
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let hash_code = ShardCount::hash(&key);
        let shard_index = ShardCount::shard_index(self.shard_count, hash_code);
        let mut mutex_guard = self.custodian.guard_at(shard_index);
        let entry = mutex_guard.entry(
            hash_code.0,
            |entry| entry.0 == key,
            |entry| ShardCount::hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => {
                let ((old_key, old_value), vacant) = occupied.remove();
                vacant.insert((old_key, value));
                Some(old_value)
            }
            Entry::Vacant(vacant) => {
                vacant.insert((key, value));
                None
            }
        }
    }
    #[must_use]
    pub fn get_with<R>(&self, key: &K, transform: impl FnOnce(&V) -> R) -> Option<R> {
        let hash_code = ShardCount::hash(&key);
        let shard_index = ShardCount::shard_index(self.shard_count, hash_code);
        let mut mutex_guard = self.custodian.guard_at(shard_index);
        let entry = mutex_guard.entry(
            hash_code.0,
            |entry| entry.0 == *key,
            |entry| ShardCount::hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => Some(transform(&occupied.get().1)),
            Entry::Vacant(_) => None,
        }
    }
    #[must_use]
    pub fn get_all_with<R>(
        &self,
        keys: impl IntoIterator<Item = K>,
        transform: impl Fn(&K, &V) -> R,
    ) -> Vec<Option<R>> {
        let indexed_keys = ShardCount::indexes(self.shard_count, keys, |k| k);
        let mutex_guards = self.custodian.guards(indexed_keys.bitmask);
        let mut result = Vec::with_capacity(indexed_keys.indexed.len());
        for indexed_key in &indexed_keys.indexed {
            let value_ref = indexed_key.value_ref(&mutex_guards);
            let result_value = value_ref.map(|v| transform(&indexed_key.3, v));
            result.push(result_value);
        }
        result
    }
    #[must_use]
    pub fn fold<T, R>(
        &self,
        initial: R,
        convert: impl Fn(&K, &V) -> Option<T>,
        accumulate: impl Fn(R, T) -> R,
    ) -> R {
        self.custodian
            .all_guards()
            .iter()
            .flat_map(|guard| guard.1.iter())
            .filter_map(|(key, value)| convert(key, value))
            .fold(initial, accumulate)
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        let hash_code = ShardCount::hash(&key);
        let shard_index = ShardCount::shard_index(self.shard_count, hash_code);
        let mut mutex_guard = self.custodian.guard_at(shard_index);
        let entry = mutex_guard.entry(
            hash_code.0,
            |entry| entry.0 == *key,
            |entry| ShardCount::hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(occupied) => {
                let ((_, old_value), _) = occupied.remove();
                Some(old_value)
            }
            Entry::Vacant(_) => None,
        }
    }
    pub fn remove_if(&self, condition: impl Fn(&K, &V) -> bool) {
        let mutex_guards = self.custodian.all_guards();
        for (_, mut mutex_guard) in mutex_guards {
            mutex_guard.retain(|entry| !condition(&entry.0, &entry.1))
        }
    }
    pub fn retain(&self, condition: impl Fn(&K, &V) -> bool) {
        let mutex_guards = self.custodian.all_guards();
        for (_, mut mutex_guard) in mutex_guards {
            mutex_guard.retain(|entry| condition(&entry.0, &entry.1))
        }
    }
    pub fn retain_only(&self, keys: impl IntoIterator<Item = K>) {
        let keys: Vec<K> = keys.into_iter().collect();
        let mutex_guards = self.custodian.all_guards();
        for (_, mut mutex_guard) in mutex_guards {
            mutex_guard.retain(|entry| keys.contains(&entry.0));
        }
    }
    pub fn retain_where(
        &self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V) -> bool,
    ) {
        let keys: Vec<K> = keys.into_iter().collect();
        let mutex_guards = self.custodian.all_guards();
        for (_, mut mutex_guard) in mutex_guards {
            mutex_guard.retain(|entry| keys.contains(&entry.0) && condition(&entry.0, &entry.1));
        }
    }

    #[must_use]
    pub fn transaction<'txmap>(&'txmap self) -> TxStemBuilder<'txmap, K, V> {
        TxStemBuilder {
            custodian: &self.custodian,
        }
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
    V: Copy,
{
    #[must_use]
    pub fn get_copied(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| *v)
    }
    #[must_use]
    pub fn get_all_copied(&self, keys: impl IntoIterator<Item = K>) -> Vec<Option<V>> {
        self.get_all_with(keys, |_k, v| *v)
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    #[must_use]
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| v.clone())
    }
    #[must_use]
    pub fn get_all_cloned(&self, keys: impl IntoIterator<Item = K>) -> Vec<Option<V>> {
        self.get_all_with(keys, |_k, v| v.clone())
    }
}
