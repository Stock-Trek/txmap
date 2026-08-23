use crate::{
    custodian::Custodian,
    hasher::DefaultBuildHasher,
    immediate::tx_builder::ImmediateTxBuilder,
    indexer::Indexer,
    iter::{Drain, Iter, Keys, Values},
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    multi_shard_ops::MultiShardOps,
    new_types::ShardCount,
    new_types::ShardIndex,
    prepared::{
        schema::TxSchema,
        tx_builder::{PreparedBuilderPhase, PreparedTxBuilder},
    },
    shard_ops::ShardOps,
    tx_map_builder::TxMapBuilder,
};
use crossbeam_utils::CachePadded;
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
///
/// The key type is only required to be `Clone + Hash + Eq` (in addition to
/// the `LockPolicy` and `BuildHasher` bounds below) by the individual
/// operations; the map type itself can be named for any `K`.
pub struct TxMap<K, V, L = MutexPolicy, S = DefaultBuildHasher>
where
    L: LockPolicy,
    S: BuildHasher,
{
    pub(crate) shard_count: ShardCount,
    pub(crate) custodian: Custodian<K, V, L>,
    pub(crate) indexer: Indexer<S>,
}

impl<K, V> TxMap<K, V, MutexPolicy, DefaultBuildHasher> {
    #[must_use]
    /// Creates an empty `TxMap` with default configuration.
    ///
    /// Equivalent to `TxMap::default()`. Uses 32 shards, `MutexPolicy`,
    /// and the default hasher.
    pub fn new() -> TxMap<K, V, MutexPolicy, DefaultBuildHasher> {
        TxMap::default()
    }
}

impl<K, V> Default for TxMap<K, V, MutexPolicy, DefaultBuildHasher> {
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
    /// Reads the value for `key`, inserting `value` if the key is absent and returns a transformated value.
    ///
    /// If the key is present, the existing value is transformed and returned and
    /// `value` is discarded. If the key is absent, `value` is inserted,
    /// transformed and returned.
    #[must_use]
    pub fn get_with_or_insert<R>(&self, key: &K, transform: impl FnOnce(&V) -> R, value: V) -> R {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        let value =
            ShardOps::get_or_insert::<K, V, S>(&mut shard, hash_code, key, value, &self.indexer);
        transform(value)
    }

    /// Returns the value for `key`, inserting a generated value if the key is
    /// absent and returns a transformed value.
    ///
    /// If the key is present, the existing value is transformed and returned and
    /// `value_generator` is never called. If the key is absent,
    /// `value_generator` is called with `key`, its result is inserted, and a
    /// transformed value is returned.
    #[must_use]
    pub fn get_with_or_insert_with<R>(
        &self,
        key: &K,
        transform: impl FnOnce(&V) -> R,
        value_generator: impl FnOnce(&K) -> V,
    ) -> R {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        let value = ShardOps::get_or_insert_with::<K, V, S>(
            &mut shard,
            hash_code,
            key,
            |k| value_generator(k),
            &self.indexer,
        );
        transform(value)
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
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Hash + Eq,
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
        let hash_code = self.indexer.hash(&key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::insert::<K, V, S>(&mut shard, hash_code, key, value, &self.indexer)
    }

    /// Inserts a value only if the key is absent.
    ///
    /// The value is lazily created by `value_generator`. Returns `true`
    /// if the insertion succeeded (key was absent).
    pub fn insert_with_if_absent(&self, key: K, value_generator: impl FnOnce() -> V) -> bool {
        let hash_code = self.indexer.hash(&key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::insert_if_absent::<K, V, S>(
            &mut shard,
            hash_code,
            key,
            |_key| value_generator(),
            &self.indexer,
        )
    }

    /// Mutates an existing value in-place.
    ///
    /// Does nothing if the key is absent. Returns `true` if the key existed
    /// and the mutation was applied.
    pub fn modify(&self, key: &K, mutate: impl FnOnce(&K, &mut V)) -> bool {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::modify::<K, V>(&mut shard, hash_code, key, mutate)
    }

    /// Removes a key and returns its value.
    ///
    /// Returns `None` if the key was absent.
    pub fn remove(&self, key: &K) -> Option<V> {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::remove_entry::<K, V>(&mut shard, hash_code, key).map(|removed| removed.1)
    }

    /// Removes a key only if `condition` is satisfied.
    ///
    /// Returns the value if it was removed, `None` otherwise.
    pub fn remove_if(&self, key: &K, condition: impl FnOnce(&K, &V) -> bool) -> Option<V> {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::remove_if::<K, V>(&mut shard, hash_code, key, condition)
    }

    /// Returns `true` if the map contains the given key.
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let shard = self.custodian.read_guard_at(shard_index);
        shard.find(hash_code.0, |entry| entry.0 == *key).is_some()
    }

    /// Removes a key and returns both the key and its value.
    ///
    /// Returns `None` if the key was absent.
    #[must_use]
    pub fn remove_entry(&self, key: &K) -> Option<(K, V)> {
        let hash_code = self.indexer.hash(key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::remove_entry::<K, V>(&mut shard, hash_code, key)
    }

    /// Updates or removes an entry based on `transform`.
    ///
    /// If `transform` returns `Some(v)` the entry is inserted or replaced;
    /// if it returns `None` the entry is removed.
    pub fn update(&self, key: K, transform: impl FnOnce(&K, Option<&V>) -> Option<V>) {
        let hash_code = self.indexer.hash(&key);
        let shard_index = Indexer::<S>::shard_index(self.shard_count, hash_code);
        let mut shard = self.custodian.write_guard_at(shard_index);
        ShardOps::update::<K, V, S>(&mut shard, hash_code, key, transform, &self.indexer)
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Hash,
    L: LockPolicy,
    S: BuildHasher,
{
    /// Reserves capacity for at least `additional` more entries.
    ///
    /// The additional capacity is distributed evenly across all shards.
    pub fn reserve(&self, additional: usize) {
        let per_shard = additional.div_ceil(self.shard_count.0 as usize);
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut guard = self.custodian.write_guard_at(ShardIndex(shard_index));
            guard.reserve(per_shard, |entry| self.indexer.hash(&entry.0).0);
            guards.push(guard);
        }
    }

    /// Tries to reserve capacity for at least `additional` more entries.
    ///
    /// The additional capacity is distributed evenly across all shards.
    pub fn try_reserve(&self, additional: usize) -> Result<(), crate::result::TryReserveError> {
        let per_shard = additional.div_ceil(self.shard_count.0 as usize);
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut guard = self.custodian.write_guard_at(ShardIndex(shard_index));
            guard
                .try_reserve(per_shard, |entry| self.indexer.hash(&entry.0).0)
                .map_err(|error| match error {
                    hashbrown::TryReserveError::CapacityOverflow => {
                        crate::result::TryReserveError::CapacityOverflow
                    }
                    hashbrown::TryReserveError::AllocError { layout } => {
                        crate::result::TryReserveError::AllocError { layout }
                    }
                })?;
            guards.push(guard);
        }
        Ok(())
    }

    /// Shrinks the capacity of all shards as much as possible.
    pub fn shrink_to_fit(&self) {
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut guard = self.custodian.write_guard_at(ShardIndex(shard_index));
            guard.shrink_to_fit(|entry| self.indexer.hash(&entry.0).0);
            guards.push(guard);
        }
    }

    /// Shrinks the capacity of all shards to a lower bound.
    ///
    /// The lower bound is distributed evenly across all shards.
    pub fn shrink_to(&self, min_capacity: usize) {
        let per_shard = min_capacity.div_ceil(self.shard_count.0 as usize);
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut guard = self.custodian.write_guard_at(ShardIndex(shard_index));
            guard.shrink_to(per_shard, |entry| self.indexer.hash(&entry.0).0);
            guards.push(guard);
        }
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    L: LockPolicy,
    S: BuildHasher,
{
    #[must_use]
    /// Starts building an immediate (one-shot) transaction.
    ///
    /// The type parameter `STATE` defines the mutable working state
    /// for the transaction and must implement `Default`.
    pub fn immediate_tx<'tx, STATE>(&'tx self) -> ImmediateTxBuilder<'tx, K, V, L, S, STATE>
    where
        K: 'tx,
        V: 'tx,
        STATE: 'tx,
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
    /// `_schema` is a schema constant created via the [`tx_schema`](macro@crate::tx_schema) macro.
    /// The returned builder can be turned into a [`PreparedTransaction`](crate::prepared::transaction::PreparedTransaction)
    /// that can be executed many times with different keys/parameters.
    ///
    /// The [`tx_schema`](macro@crate::tx_schema) macro also generates a specialised
    /// `prepared_tx` function for each schema (e.g. `transfer_prepared_tx`) that
    /// infers the schema from the map, so the schema argument is optional.
    pub fn prepared_tx<'tx, SCHEMA>(
        &'tx self,
        _schema: &SCHEMA,
    ) -> PreparedTxBuilder<'tx, SCHEMA, V, L, S, PreparedBuilderPhase>
    where
        SCHEMA: TxSchema<Key = K> + 'tx,
        K: 'tx,
        V: 'tx,
        L: 'tx,
        S: 'tx,
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
    ///
    /// Acquires each shard's write lock lazily, one at a time, and holds all
    /// acquired locks until every shard has been cleared so the operation is
    /// a consistent snapshot.
    pub fn clear(&self) {
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut write_guard = self.custodian.write_guard_at(ShardIndex(shard_index));
            write_guard.clear();
            guards.push(write_guard);
        }
    }

    /// Returns the total number of entries across all shards.
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        // Acquire each shard's read lock lazily, one at a time, and hold all
        // acquired locks until the count is complete so the result is a
        // consistent snapshot.
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let guard = self.custodian.read_guard_at(ShardIndex(shard_index));
            total_length += guard.len();
            guards.push(guard);
        }
        total_length
    }

    /// Returns `true` if the map contains no entries.
    ///
    /// Shards are checked one at a time and the scan short-circuits at the
    /// first non-empty shard, so an occupied map only locks as many shards
    /// as needed instead of locking all of them.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let guard = self.custodian.read_guard_at(ShardIndex(shard_index));
            if !guard.is_empty() {
                return false;
            }
            guards.push(guard);
        }
        true
    }

    /// Returns the total capacity of all shards.
    ///
    /// This is an approximation: each shard allocates capacity in
    /// implementation-defined increments, so the returned value may exceed
    /// the number of entries the map can hold without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        let mut total_capacity = 0;
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let guard = self.custodian.read_guard_at(ShardIndex(shard_index));
            total_capacity += guard.capacity();
            guards.push(guard);
        }
        total_capacity
    }

    /// Returns the hasher builder used by this map.
    #[must_use]
    pub fn hasher(&self) -> &S {
        self.indexer.hasher_builder()
    }

    #[must_use]
    /// Folds over all entries in the map.
    ///
    /// Each entry is optionally converted to an intermediate value via
    /// `convert`, then accumulated with `accumulate`. Iteration order
    /// is not guaranteed. Read locks are acquired lazily, one shard at a
    /// time, and held until the fold completes.
    pub fn fold<T, R>(
        &self,
        initial: R,
        convert: impl Fn(&K, &V) -> Option<T>,
        accumulate: impl Fn(R, T) -> R,
    ) -> R {
        let mut result = initial;
        // Acquire each shard's read lock lazily, one at a time, and hold all
        // acquired locks until the fold is complete.
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let guard = self.custodian.read_guard_at(ShardIndex(shard_index));
            for (key, value) in guard.iter() {
                if let Some(intermediate) = convert(key, value) {
                    result = accumulate(result, intermediate);
                }
            }
            guards.push(guard);
        }
        result
    }

    #[must_use]
    /// Returns an iterator over all key-value pairs.
    ///
    /// Acquires read locks lazily, one shard at a time, as iteration
    /// progresses. Acquired locks are held until the iterator is dropped.
    pub fn iter(&self) -> Iter<'_, K, V, L> {
        Iter::new(&self.custodian)
    }

    #[must_use]
    /// Returns an iterator over all the keys.
    ///
    /// Acquires read locks on all shards for the duration of iteration.
    pub fn keys(&self) -> Keys<'_, K, V, L> {
        Keys(self.iter())
    }

    #[must_use]
    /// Returns an iterator over all the values.
    ///
    /// Acquires read locks on all shards for the duration of iteration.
    pub fn values(&self) -> Values<'_, K, V, L> {
        Values(self.iter())
    }

    /// Removes all entries and returns an iterator over them.
    ///
    /// Entries are removed as the iterator is consumed; dropping the
    /// iterator without fully consuming it removes all remaining entries.
    /// Acquires write locks lazily, one shard at a time, as iteration
    /// progresses. Acquired locks are held until the iterator is dropped.
    pub fn drain(&self) -> Drain<'_, K, V, L> {
        Drain::new(&self.custodian)
    }

    /// Consumes the map and returns an iterator over its keys.
    #[must_use]
    pub fn into_keys(self) -> std::vec::IntoIter<K> {
        self.drain()
            .map(|(key, _)| key)
            .collect::<Vec<K>>()
            .into_iter()
    }

    /// Consumes the map and returns an iterator over its values.
    #[must_use]
    pub fn into_values(self) -> std::vec::IntoIter<V> {
        self.drain()
            .map(|(_, value)| value)
            .collect::<Vec<V>>()
            .into_iter()
    }

    /// Retains only entries satisfying `condition`.
    ///
    /// Removes all entries for which `condition` returns `false`. Acquires
    /// each shard's write lock lazily, one at a time, and holds all acquired
    /// locks until every shard has been processed.
    pub fn retain(&self, condition: impl Fn(&K, &V) -> bool) {
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let mut shard = self.custodian.write_guard_at(ShardIndex(shard_index));
            shard.retain(|entry| condition(&entry.0, &entry.1));
            guards.push(shard);
        }
    }
}

impl<K, V, L, S> TxMap<K, V, L, S>
where
    K: Hash + Eq,
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
    K: Hash + Eq,
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
    K: Clone,
    V: Clone,
    L: LockPolicy,
    S: Clone + BuildHasher,
{
    fn clone(&self) -> Self {
        let shard_count = self.shard_count;
        let mut shards = Vec::with_capacity(shard_count.0 as usize);
        // Acquire each shard's read lock lazily, one at a time, and hold all
        // acquired locks until every shard has been cloned.
        let mut guards = Vec::new();
        for shard_index in 0..self.shard_count.0 {
            let shard = self.custodian.read_guard_at(ShardIndex(shard_index));
            let cloned_shard = shard.clone();
            shards.push(CachePadded::new(L::new(cloned_shard)));
            guards.push(shard);
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

impl<K, V, L, S> PartialEq for TxMap<K, V, L, S>
where
    K: Hash + Eq,
    V: PartialEq,
    L: LockPolicy,
    S: BuildHasher,
{
    /// Two maps are equal if they contain the same key-value pairs.
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        self.iter().all(|(key, value)| {
            other
                .get_with(key, |other_value| other_value == value)
                .unwrap_or(false)
        })
    }
}

impl<K, V, L, S> Eq for TxMap<K, V, L, S>
where
    K: Hash + Eq,
    V: Eq,
    L: LockPolicy,
    S: BuildHasher,
{
}

impl<K, V, L, S> std::fmt::Debug for TxMap<K, V, L, S>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
    L: LockPolicy,
    S: BuildHasher,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K, V, L, S> Extend<(K, V)> for TxMap<K, V, L, S>
where
    K: Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<'a, K, V, L, S> Extend<(&'a K, &'a V)> for TxMap<K, V, L, S>
where
    K: Clone + Hash + Eq + 'a,
    V: Clone + 'a,
    L: LockPolicy,
    S: BuildHasher,
{
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key.clone(), value.clone());
        }
    }
}

impl<K, V, L, S> FromIterator<(K, V)> for TxMap<K, V, L, S>
where
    K: Hash + Eq,
    L: LockPolicy,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map: TxMap<K, V, L, S> = TxMapBuilder::default()
            .with_lock_policy::<L>()
            .with_hasher(S::default())
            .build();
        map.extend(iter);
        map
    }
}

impl<K, V, L, S, const N: usize> From<[(K, V); N]> for TxMap<K, V, L, S>
where
    K: Hash + Eq,
    L: LockPolicy,
    S: BuildHasher + Default,
{
    fn from(array: [(K, V); N]) -> Self {
        let map: TxMap<K, V, L, S> = TxMapBuilder::default()
            .with_lock_policy::<L>()
            .with_hasher(S::default())
            .build();
        for (key, value) in array {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V, L, S> IntoIterator for TxMap<K, V, L, S>
where
    L: LockPolicy,
    S: BuildHasher,
{
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    /// Consumes the map and iterates over its entries.
    ///
    /// Unlike `std::collections::HashMap`, iteration is eager: all entries
    /// are drained into a buffer before the map is dropped.
    fn into_iter(self) -> Self::IntoIter {
        self.drain().collect::<Vec<(K, V)>>().into_iter()
    }
}
