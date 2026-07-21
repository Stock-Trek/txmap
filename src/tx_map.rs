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
    pub fn get_with<T, R>(&self, key: &K, transform: T) -> Option<R>
    where
        T: FnOnce(&V) -> R,
    {
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
    pub fn fold<T, R, C, A>(&self, initial: R, convert: C, accumulate: A) -> R
    where
        C: Fn(&K, &V) -> Option<T>,
        A: Fn(R, T) -> R,
    {
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
    pub fn retain_only<I>(&self, keys: impl IntoIterator<Item = K>)
    where
        I: IntoIterator<Item = K>,
    {
        let keys: Vec<K> = keys.into_iter().collect();
        let mutex_guards = self.custodian.all_guards();
        for (_, mut mutex_guard) in mutex_guards {
            mutex_guard.retain(|entry| keys.contains(&entry.0));
        }
    }
    pub fn retain_where<I, C>(
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
}
