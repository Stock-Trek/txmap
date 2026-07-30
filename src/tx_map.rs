use crate::{
    custodian::Custodian,
    immediate::tx_builder::ImmediateTxBuilder,
    indexer::Indexer,
    iter::Iter,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    multi_shard_ops::MultiShardOps,
    new_types::ShardCount,
    prepared::{
        schema::{TxKeys, TxSchema},
        tx_builder::{PreparedBuilderPhase, PreparedTxBuilder},
    },
    shard_ops::ShardOps,
    tx_map_builder::TxMapBuilder,
};
use std::hash::Hash;

pub struct TxMap<K, V, L = MutexPolicy>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
{
    pub(crate) shard_count: ShardCount,
    pub(crate) custodian: Custodian<K, V, L>,
}

impl<K, V> TxMap<K, V>
where
    K: Clone + Hash + Eq,
{
    #[must_use]
    pub fn new() -> TxMap<K, V, MutexPolicy> {
        TxMap::default()
    }
}

impl<K, V> Default for TxMap<K, V, MutexPolicy>
where
    K: Clone + Hash + Eq,
{
    fn default() -> Self {
        TxMapBuilder::default().build()
    }
}

impl<K, V, L> TxMap<K, V, L>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
{
    #[must_use]
    pub fn get_with<R>(&self, key: &K, transform: impl FnOnce(&V) -> R) -> Option<R> {
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let shard = self.custodian.read_guard_at(shard_index);
        let entry = shard.find(hash_code.0, |entry| entry.0 == *key);
        entry.map(|e| transform(&e.1))
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let tx_key = Indexer::indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::insert::<K, V>(&mut shard, &tx_key, value)
    }
    pub fn insert_with_if_absent(&self, key: K, value_generator: impl FnOnce() -> V) -> bool {
        let tx_key = Indexer::indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::insert_if_absent::<K, V>(&mut shard, &tx_key, value_generator)
    }
    pub fn modify(&self, key: &K, mutate: impl FnOnce(&K, &mut V)) -> bool {
        let tx_key = Indexer::indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::modify::<K, V>(&mut shard, &tx_key, mutate)
    }
    pub fn move_value(&self, key_from: &K, key_to: K) {
        let tx_key_from = Indexer::indexed_key(self.shard_count, key_from.clone());
        let tx_key_to = Indexer::indexed_key(self.shard_count, key_to);
        let mut shards = self
            .custodian
            .write_guards(tx_key_from.shard_index.bitmask() | tx_key_to.shard_index.bitmask());
        MultiShardOps::move_value::<K, V, L>(&mut shards, &tx_key_from, &tx_key_to);
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        let tx_key = Indexer::indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::remove_entry::<K, V>(&mut shard, &tx_key).map(|removed| removed.1)
    }
    pub fn remove_if(&self, key: &K, condition: impl FnOnce(&K, &V) -> bool) -> Option<V> {
        let tx_key = Indexer::indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::remove_if::<K, V>(&mut shard, &tx_key, condition)
    }
    pub fn swap_value(&self, key_a: K, key_b: K) {
        let tx_key_a = Indexer::indexed_key(self.shard_count, key_a);
        let tx_key_b = Indexer::indexed_key(self.shard_count, key_b);
        let mut shards = self
            .custodian
            .write_guards(tx_key_a.shard_index.bitmask() | tx_key_b.shard_index.bitmask());
        MultiShardOps::swap_value::<K, V, L>(&mut shards, &tx_key_a, &tx_key_b);
    }
    pub fn update(&self, key: K, transform: impl FnOnce(&K, Option<&V>) -> Option<V>) {
        let tx_key = Indexer::indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::update::<K, V>(&mut shard, &tx_key, transform)
    }

    #[must_use]
    pub fn immediate_tx<'tx, STATE>(&'tx self) -> ImmediateTxBuilder<'tx, K, V, L, STATE>
    where
        K: 'tx,
        V: 'tx,
        STATE: Default + 'tx,
    {
        ImmediateTxBuilder {
            custodian: &self.custodian,
            guards: Vec::new(),
            ops: Vec::new(),
            _phase: std::marker::PhantomData,
        }
    }
    #[must_use]
    pub fn prepared_tx<'tx, SCHEMA, RAW, KEYS, PARAMS, STATE>(
        &'tx self,
        _schema: &SCHEMA,
    ) -> PreparedTxBuilder<'tx, K, V, L, KEYS, PARAMS, STATE, PreparedBuilderPhase>
    where
        K: 'tx,
        V: 'tx,
        SCHEMA: TxSchema<K, Keys = RAW, IndexedKeys = KEYS, Params = PARAMS, State = STATE> + 'tx,
        RAW: TxKeys<K, KEYS> + 'tx,
        KEYS: 'tx,
        PARAMS: 'tx,
        STATE: Default + 'tx,
    {
        PreparedTxBuilder {
            custodian: &self.custodian,
            guards: Vec::new(),
            ops: Vec::new(),
            _phase: std::marker::PhantomData,
        }
    }

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
    #[must_use]
    pub fn iter(&self) -> Iter<'_, K, V, L> {
        let guards = self.custodian.all_read_guards();
        let remaining: usize = guards.iter().map(|(_, guard)| guard.len()).sum();
        Iter {
            _guards: guards,
            shard_index: 0,
            bucket_index: 0,
            shard_count: self.shard_count.0,
            remaining,
        }
    }
    pub fn retain(&self, condition: impl Fn(&K, &V) -> bool) {
        let shards = self.custodian.all_write_guards();
        for (_, mut shard) in shards {
            shard.retain(|entry| condition(&entry.0, &entry.1))
        }
    }
}

impl<K, V, L> TxMap<K, V, L>
where
    K: Clone + Hash + Eq,
    V: Copy,
    L: LockPolicy,
{
    #[must_use]
    pub fn get_copied(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| *v)
    }
}

impl<K, V, L> TxMap<K, V, L>
where
    K: Clone + Hash + Eq,
    V: Clone,
    L: LockPolicy,
{
    #[must_use]
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| v.clone())
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
