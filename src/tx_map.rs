use crate::{
    custodian::Custodian,
    indexer::Indexer,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::{BitMask, ShardCount},
    params::{TxKeys, TxSchema},
    prepared_transaction_builder::{BuilderPhase, PreparedTransactionBuilder},
    result::MISSING_LOCK_GUARD_ERROR,
    shards::Shards,
};
use hashbrown::hash_table::Entry;
use std::hash::Hash;

pub struct TxMap<K, V, L = MutexPolicy>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    shard_count: ShardCount,
    custodian: Custodian<K, V, L>,
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    #[must_use]
    pub fn new(shards: Shards) -> Self {
        let shard_count = shards.into();
        Self {
            shard_count,
            custodian: Custodian::new(shard_count),
        }
    }
    #[must_use]
    pub fn with_lock_policy<L>(shards: Shards) -> TxMap<K, V, L>
    where
        L: LockPolicy,
    {
        let shard_count = shards.into();
        TxMap::<K, V, L> {
            shard_count,
            custodian: Custodian::new(shard_count),
        }
    }
}

impl<K, V, L> TxMap<K, V, L>
where
    K: Hash + Eq,
    L: LockPolicy,
{
    pub fn clear(&self) {
        for mut write_guard in self.custodian.all_write_guards() {
            write_guard.1.clear();
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        for read_guard in self.custodian.all_read_guards() {
            total_length += read_guard.1.len();
        }
        total_length
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let mut write_guard = self.custodian.write_guard_at(shard_index);
        let entry = write_guard.entry(
            hash_code.0,
            |entry| entry.0 == key,
            |entry| Indexer::hash(&entry.0).0,
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
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let read_guard = self.custodian.read_guard_at(shard_index);
        let entry = read_guard.find(hash_code.0, |entry| entry.0 == *key);
        entry.map(|e| transform(&e.1))
    }
    #[must_use]
    pub fn get_all_with<R>(
        &self,
        keys: impl IntoIterator<Item = K>,
        transform: impl Fn(&K, &V) -> R,
    ) -> Vec<Option<R>> {
        let indexed_keys = Indexer::all_indexed_keys(self.shard_count, keys, |k| k);
        let bitmask = indexed_keys
            .iter()
            .fold(BitMask::ZERO, |bitmask, indexed_key| {
                bitmask | indexed_key.shard_index.bitmask()
            });
        let read_guards = self.custodian.read_guards(bitmask);
        let mut result = Vec::with_capacity(indexed_keys.len());
        for indexed_key in &indexed_keys {
            let guard = read_guards
                .get(indexed_key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR);
            let result_value = guard
                .find(indexed_key.hash_code.0, |k| k.0 == indexed_key.key)
                .map(|entry| transform(&entry.0, &entry.1));
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
            .all_read_guards()
            .iter()
            .flat_map(|guard| guard.1.iter())
            .filter_map(|(key, value)| convert(key, value))
            .fold(initial, accumulate)
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let mut write_guard = self.custodian.write_guard_at(shard_index);
        let entry = write_guard.entry(
            hash_code.0,
            |entry| entry.0 == *key,
            |entry| Indexer::hash(&entry.0).0,
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
        let write_guards = self.custodian.all_write_guards();
        for (_, mut mutex_guard) in write_guards {
            mutex_guard.retain(|entry| !condition(&entry.0, &entry.1))
        }
    }
    pub fn retain(&self, condition: impl Fn(&K, &V) -> bool) {
        let write_guards = self.custodian.all_write_guards();
        for (_, mut mutex_guard) in write_guards {
            mutex_guard.retain(|entry| condition(&entry.0, &entry.1))
        }
    }
    pub fn retain_only(&self, keys: impl IntoIterator<Item = K>) {
        let keys: Vec<K> = keys.into_iter().collect();
        let write_guards = self.custodian.all_write_guards();
        for (_, mut mutex_guard) in write_guards {
            mutex_guard.retain(|entry| keys.contains(&entry.0));
        }
    }
    pub fn retain_where(
        &self,
        keys: impl IntoIterator<Item = K>,
        condition: impl Fn(&K, &V) -> bool,
    ) {
        let keys: Vec<K> = keys.into_iter().collect();
        let write_guards = self.custodian.all_write_guards();
        for (_, mut mutex_guard) in write_guards {
            mutex_guard.retain(|entry| keys.contains(&entry.0) && condition(&entry.0, &entry.1));
        }
    }

    #[must_use]
    pub fn prepare_transaction<'tx, SCHEMA, RAW, KEYS, PARAMS, STATE>(
        &'tx self,
        _schema: &SCHEMA,
    ) -> PreparedTransactionBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, BuilderPhase>
    where
        K: 'tx,
        V: 'tx,
        SCHEMA: TxSchema<K, Keys = RAW, IndexedKeys = KEYS, Params = PARAMS, State = STATE>,
        RAW: TxKeys<K, KEYS> + 'tx,
        KEYS: 'tx,
        PARAMS: 'tx,
        STATE: Default + 'tx,
    {
        PreparedTransactionBuilder {
            custodian: &self.custodian,
            guards: Vec::new(),
            ops: Vec::new(),
            _phase: std::marker::PhantomData,
        }
    }

    // #[must_use]
    // pub fn transaction<'tx, STATE>(&'tx self, initial_state: STATE) -> TxResult<STATE>
    // where
    //     K: 'tx,
    //     V: 'tx,
    //     STATE: Default + 'tx,
    // {
    //     // TODO
    // }
}

impl<K, V, L> TxMap<K, V, L>
where
    K: Hash + Eq,
    V: Copy,
    L: LockPolicy,
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

impl<K, V, L> TxMap<K, V, L>
where
    K: Hash + Eq,
    V: Clone,
    L: LockPolicy,
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

impl<K, V, L> Clone for TxMap<K, V, L>
where
    K: Clone + Hash + Eq,
    V: Clone,
    L: LockPolicy,
{
    fn clone(&self) -> Self {
        let shard_count = self.shard_count;
        let mut shards = Vec::with_capacity(shard_count.0 as usize);
        for (_, shard) in self.custodian.all_read_guards() {
            let cloned_shard = shard.clone();
            shards.push(L::new(cloned_shard));
        }
        let custodian = Custodian {
            shard_count,
            shards,
        };
        TxMap {
            shard_count,
            custodian,
        }
    }
}
