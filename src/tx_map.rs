use crate::{
    custodian::Custodian,
    hasher::DefaultBuildHasher,
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
use std::hash::{BuildHasher, Hash};

/// A concurrent transactional hash map.
///
/// Entries are distributed across shards, each protected by a configurable
/// lock policy. All mutating operations are atomic per shard; multi-shard
/// operations (e.g. [`move_value`](TxMap::move_value)) acquire locks on
/// all involved shards to remain atomic.
///
/// The map supports both immediate one-shot transactions and prepared
/// re-usable transactions. Guard-based preconditions can veto a transaction.
pub struct TxMap<K, V, L = MutexPolicy, S = DefaultBuildHasher>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
{
    pub(crate) shard_count: ShardCount,
    pub(crate) custodian: Custodian<K, V, L>,
    pub(crate) indexer: Indexer<S>,
}

impl<K, V> TxMap<K, V, MutexPolicy, DefaultBuildHasher>
where
    K: Clone + Hash + Eq,
{
    #[must_use]
    /// Creates an empty `TxMap` with default configuration.
    ///
    /// Equivalent to `TxMap::default()`. Uses 32 shards, `MutexPolicy`,
    /// and the default hasher.
    pub fn new() -> TxMap<K, V, MutexPolicy, DefaultBuildHasher> {
        TxMap::default()
    }
}

impl<K, V> Default for TxMap<K, V, MutexPolicy, DefaultBuildHasher>
where
    K: Clone + Hash + Eq,
{
    fn default() -> Self {
        TxMapBuilder::default().build()
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
{
    #[must_use]
    /// Reads the value for `key` and applies a transformation.
    ///
    /// Acquires only a read lock on the relevant shard. Returns `None`
    /// if the key is absent.
    pub fn get_with<R>(&self, key: &K, transform: impl FnOnce(&V) -> R) -> Option<R> {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let shard = self.custodian.read_guard_at(shard_index);
        let entry = shard.find(hash_code.0, |entry| entry.0 == *key);
        entry.map(|e| transform(&e.1))
    }

    /// Inserts a key-value pair.
    ///
    /// Returns the previous value if the key already existed.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let tx_key = self.indexer.indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::insert::<K, V, S>(&mut shard, &tx_key, value, &self.indexer)
    }

    /// Inserts a value only if the key is absent.
    ///
    /// The value is lazily created by `value_generator`. Returns `true`
    /// if the insertion succeeded (key was absent).
    pub fn insert_with_if_absent(&self, key: K, value_generator: impl FnOnce() -> V) -> bool {
        let tx_key = self.indexer.indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::insert_if_absent::<K, V, S>(&mut shard, &tx_key, value_generator, &self.indexer)
    }

    /// Mutates an existing value in-place.
    ///
    /// Does nothing if the key is absent. Returns `true` if the key existed
    /// and the mutation was applied.
    pub fn modify(&self, key: &K, mutate: impl FnOnce(&K, &mut V)) -> bool {
        let tx_key = self.indexer.indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::modify::<K, V>(&mut shard, &tx_key, mutate)
    }

    /// Moves a value from one key to another atomically.
    ///
    /// If the source key was absent the destination key is removed.
    /// Acquires write locks on both shards involved.
    pub fn move_value(&self, key_from: K, key_to: K) {
        let tx_key_from = self.indexer.indexed_key(self.shard_count, key_from);
        let tx_key_to = self.indexer.indexed_key(self.shard_count, key_to);
        let mut shards = self
            .custodian
            .write_guards(tx_key_from.shard_index.bitmask() | tx_key_to.shard_index.bitmask());
        MultiShardOps::move_value::<K, V, L, S>(
            &mut shards,
            &tx_key_from,
            &tx_key_to,
            &self.indexer,
        );
    }

    /// Removes a key and returns its value.
    ///
    /// Returns `None` if the key was absent.
    pub fn remove(&self, key: &K) -> Option<V> {
        let tx_key = self.indexer.indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::remove_entry::<K, V>(&mut shard, &tx_key).map(|removed| removed.1)
    }

    /// Removes a key only if `condition` is satisfied.
    ///
    /// Returns the value if it was removed, `None` otherwise.
    pub fn remove_if(&self, key: &K, condition: impl FnOnce(&K, &V) -> bool) -> Option<V> {
        let tx_key = self.indexer.indexed_key(self.shard_count, key.clone());
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::remove_if::<K, V, S>(&mut shard, &tx_key, condition, &self.indexer)
    }

    /// Swaps the values of two keys atomically.
    ///
    /// Acquires write locks on both shards involved.
    pub fn swap_value(&self, key_a: K, key_b: K) {
        let tx_key_a = self.indexer.indexed_key(self.shard_count, key_a);
        let tx_key_b = self.indexer.indexed_key(self.shard_count, key_b);
        let mut shards = self
            .custodian
            .write_guards(tx_key_a.shard_index.bitmask() | tx_key_b.shard_index.bitmask());
        MultiShardOps::swap_value::<K, V, L, S>(&mut shards, &tx_key_a, &tx_key_b, &self.indexer);
    }

    /// Updates or removes an entry based on `transform`.
    ///
    /// If `transform` returns `Some(v)` the entry is inserted or replaced;
    /// if it returns `None` the entry is removed.
    pub fn update(&self, key: K, transform: impl FnOnce(&K, Option<&V>) -> Option<V>) {
        let tx_key = self.indexer.indexed_key(self.shard_count, key);
        let mut shard = self.custodian.write_guard_at(tx_key.shard_index);
        ShardOps::update::<K, V, S>(&mut shard, &tx_key, transform, &self.indexer)
    }

    #[must_use]
    /// Starts building an immediate (one-shot) transaction.
    ///
    /// The type parameter `STATE` defines the mutable working state
    /// for the transaction and must implement `Default`.
    pub fn immediate_tx<'tx, STATE>(&'tx self) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE>
    where
        K: 'tx,
        V: 'tx,
        STATE: Default + 'tx,
    {
        ImmediateTxBuilder {
            custodian: &self.custodian,
            indexer: &self.indexer,
            guards: Vec::new(),
            ops: Vec::new(),
            _phase: std::marker::PhantomData,
        }
    }

    #[must_use]
    /// Starts building a prepared (re-usable) transaction.
    ///
    /// `_schema` is a schema constant created via the [`tx_schema`] macro.
    /// The returned builder can be turned into a [`PreparedTransaction`]
    /// that can be executed many times with different keys/parameters.
    pub fn prepared_tx<'tx, SCHEMA, RAW, KEYS, PARAMS, STATE>(
        &'tx self,
        _schema: &SCHEMA,
    ) -> PreparedTxBuilder<'tx, K, V, L, S, KEYS, PARAMS, STATE, PreparedBuilderPhase>
    where
        K: 'tx,
        V: 'tx,
        S: 'tx,
        SCHEMA: TxSchema<K, Keys = RAW, IndexedKeys = KEYS, Params = PARAMS, State = STATE> + 'tx,
        RAW: TxKeys<K, KEYS, S> + 'tx,
        KEYS: 'tx,
        PARAMS: 'tx,
        STATE: Default + 'tx,
    {
        PreparedTxBuilder {
            custodian: &self.custodian,
            indexer: &self.indexer,
            guards: Vec::new(),
            ops: Vec::new(),
            _phase: std::marker::PhantomData,
        }
    }

    /// Removes all entries from the map.
    pub fn clear(&self) {
        for mut write_guard in self.custodian.all_write_guards() {
            write_guard.1.clear();
        }
    }

    /// Returns the total number of entries across all shards.
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        for read_guard in self.custodian.all_read_guards() {
            total_length += read_guard.1.len();
        }
        total_length
    }

    /// Returns `true` if the map contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    /// Folds over all entries in the map.
    ///
    /// Each entry is optionally converted to an intermediate value via
    /// `convert`, then accumulated with `accumulate`. Iteration order
    /// is not guaranteed.
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
    /// Returns an iterator over all key-value pairs.
    ///
    /// Acquires read locks on all shards for the duration of iteration.
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

    /// Retains only entries satisfying `condition`.
    ///
    /// Removes all entries for which `condition` returns `false`.
    pub fn retain(&self, condition: impl Fn(&K, &V) -> bool) {
        let shards = self.custodian.all_write_guards();
        for (_, mut shard) in shards {
            shard.retain(|entry| condition(&entry.0, &entry.1))
        }
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Clone + Hash + Eq,
    V: Copy,
    L: LockPolicy,
    S: BuildHasher,
{
    /// Returns a copy of the value for `key` (`V: Copy`).
    #[must_use]
    pub fn get_copied(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| *v)
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    L: LockPolicy,
    S: BuildHasher,
{
    /// Returns a clone of the value for `key` (`V: Clone`).
    #[must_use]
    pub fn get_cloned(&self, key: &K) -> Option<V> {
        self.get_with(key, |v| v.clone())
    }
}

impl<K, V, L, S> Clone for TxMap<K, V, L, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    L: LockPolicy,
    S: Clone + BuildHasher,
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
            indexer: Indexer::new(self.indexer.hasher_builder().clone()),
        }
    }
}
