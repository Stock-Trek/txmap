use crate::{
    custodian::Custodian,
    immediate::tx_builder::ImmediateTxBuilder,
    indexer::Indexer,
    iter::Iter,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::{BitMask, ShardCount},
    prepared::{
        schema::{TxKeys, TxSchema},
        tx_builder::{PreparedBuilderPhase, PreparedTxBuilder},
    },
    result::MISSING_LOCK_GUARD_ERROR,
    tx_map_builder::TxMapBuilder,
};
use hashbrown::hash_table::Entry;
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
        let read_guard = self.custodian.read_guard_at(shard_index);
        let entry = read_guard.find(hash_code.0, |entry| entry.0 == *key);
        entry.map(|e| transform(&e.1))
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
    pub fn insert_with_if_absent(&self, key: K, value_generator: impl FnOnce() -> V) -> bool {
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let mut write_guard = self.custodian.write_guard_at(shard_index);
        let entry = write_guard.entry(
            hash_code.0,
            |entry| entry.0 == key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(_occupied) => false,
            Entry::Vacant(vacant) => {
                vacant.insert((key, value_generator()));
                true
            }
        }
    }
    pub fn modify(&self, key: &K, mutate: impl FnOnce(&K, &mut V)) -> bool {
        let hash_code = Indexer::hash(&key);
        let shard_index = Indexer::shard_index(self.shard_count, hash_code);
        let mut write_guard = self.custodian.write_guard_at(shard_index);
        let entry = write_guard.entry(
            hash_code.0,
            |entry| entry.0 == *key,
            |entry| Indexer::hash(&entry.0).0,
        );
        match entry {
            Entry::Occupied(mut occupied) => {
                let entry = occupied.get_mut();
                mutate(&entry.0, &mut entry.1);
                true
            }
            Entry::Vacant(_vacant) => false,
        }
    }
    pub fn move_value(&self, key_from: &K, key_to: K) {
        let tx_key_from = Indexer::indexed_key(self.shard_count, key_from);
        let tx_key_to = Indexer::indexed_key(self.shard_count, key_to);
        let write_bitmasks = tx_key_from.shard_index.bitmask() | tx_key_from.shard_index.bitmask();
        let mut lock_guards = self.custodian.lock_guards(BitMask::ZERO, write_bitmasks);
        let removed_entry_from = lock_guards
            .write
            .get_mut(tx_key_from.shard_index.0)
            .expect(MISSING_LOCK_GUARD_ERROR)
            .find_entry(tx_key_from.hash_code.0, |entry| entry.0 == *tx_key_from.key)
            .ok()
            .map(|entry| entry.remove().0);
        if let Some((_removed_key_from, removed_value_from)) = removed_entry_from {
            let entry_to = lock_guards
                .write
                .get_mut(tx_key_to.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .entry(
                    tx_key_to.hash_code.0,
                    |entry| entry.0 == tx_key_to.key,
                    |entry| Indexer::hash(&entry.0).0,
                );
            match entry_to {
                Entry::Occupied(occupied) => {
                    occupied.replace_entry_with(|e| Some((e.0, removed_value_from)));
                }
                Entry::Vacant(vacant) => {
                    vacant.insert((tx_key_to.key, removed_value_from));
                }
            }
        } else {
            lock_guards
                .write
                .get_mut(tx_key_to.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find_entry(tx_key_to.hash_code.0, |entry| entry.0 == tx_key_to.key)
                .ok()
                .map(|entry| entry.remove().0);
        }
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
    pub fn remove_if(&self, key: &K, condition: impl FnOnce(&K, &V) -> bool) -> Option<V> {
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
                let (found_key, found_value) = occupied.get();
                if condition(found_key, found_value) {
                    Some(occupied.remove().0.1)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => None,
        }
    }
    pub fn swap_value(&self, key_a: K, key_b: K) {
        let tx_key_a = Indexer::indexed_key(self.shard_count, key_a);
        let tx_key_b = Indexer::indexed_key(self.shard_count, key_b);
        let write_bitmasks = tx_key_a.shard_index.bitmask() | tx_key_a.shard_index.bitmask();
        let mut lock_guards = self.custodian.lock_guards(BitMask::ZERO, write_bitmasks);
        let a = lock_guards.remove_entry(&tx_key_a);
        let b = lock_guards.remove_entry(&tx_key_b);
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    lock_guards.insert_with_duplicate_key(&tx_key_a, a_key, b_value);
                    lock_guards.insert_with_duplicate_key(&tx_key_b, b_key, a_value);
                }
                None => {
                    lock_guards
                        .write
                        .get_mut(tx_key_b.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .entry(
                            tx_key_b.hash_code.0,
                            |entry| entry.0 == tx_key_b.key,
                            |entry| Indexer::hash(&entry.0).0,
                        )
                        .insert((tx_key_b.key, a_value));
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    lock_guards
                        .write
                        .get_mut(tx_key_a.shard_index.0)
                        .expect(MISSING_LOCK_GUARD_ERROR)
                        .entry(
                            tx_key_a.hash_code.0,
                            |entry| entry.0 == tx_key_a.key,
                            |entry| Indexer::hash(&entry.0).0,
                        )
                        .insert((tx_key_a.key, b_value));
                }
            }
        }
    }
    pub fn update(&self, key: K, transform: impl FnOnce(&K, Option<&V>) -> Option<V>) {
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
                let (found_key, found_value) = occupied.get();
                match transform(found_key, Some(found_value)) {
                    Some(new_value) => {
                        occupied.replace_entry_with(|entry| Some((entry.0, new_value)));
                    }
                    None => {
                        occupied.remove();
                    }
                }
            }
            Entry::Vacant(vacant) => {
                if let Some(new_value) = transform(&key, None) {
                    vacant.insert((key, new_value));
                }
            }
        }
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
        let write_guards = self.custodian.all_write_guards();
        for (_, mut mutex_guard) in write_guards {
            mutex_guard.retain(|entry| condition(&entry.0, &entry.1))
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
