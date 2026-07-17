use crate::{
    custodian::Custodian,
    indexer::Indexer,
    parameterized_operation::ParameterizedOperation,
    parameterized_prerequisite::ParameterizedPrerequisite,
    result::{TxError, TxResult},
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct ParameterizedTransaction<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    owned_key: fn(&K) -> K,
    custodian: &'txmap Custodian<K, V>,
    guards_bitmask: u128,
    prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
    operations: Vec<ParameterizedOperation<K, V, P>>,
}

impl<'txmap, K, V, P> ParameterizedTransaction<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    pub fn execute(&self, params: &P) -> TxResult<()> {
        let mut guards = self.custodian.guards(self.guards_bitmask);
        for (i, prerequisite) in self.prerequisites.iter().enumerate() {
            if !self.is_prerequisite_met(prerequisite, &guards, params) {
                return Err(TxError::UnmetPrerequisite(i, prerequisite.name.clone()));
            }
        }
        for operation in &self.operations {
            let new_value = self.operation_value(operation, &guards, params);
            let guard = guards.get_mut(operation.key_index);
            let shard = guard.expect("Missing shard lock");
            match new_value {
                Some(v) => shard.insert((self.owned_key)(&operation.key), v),
                None => shard.remove(&operation.key),
            };
        }
        Ok(())
    }

    fn is_prerequisite_met(
        &self,
        prerequisite: &ParameterizedPrerequisite<K, V, P>,
        guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
        params: &P,
    ) -> bool {
        let mut values = Vec::with_capacity(prerequisite.indexed_keys.indexed.len());
        for (shard_index, key) in &prerequisite.indexed_keys.indexed {
            let guard = guards.get(*shard_index);
            let shard = guard.expect("Missing shard lock");
            let value = shard.get(key);
            values.push(value);
            if !(prerequisite.is_satisfied)(&values, params) {
                return false;
            }
        }
        true
    }
    fn operation_value(
        &self,
        operation: &ParameterizedOperation<K, V, P>,
        guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
        params: &P,
    ) -> Option<V> {
        let mut context_values = Vec::with_capacity(operation.indexed_context_keys.indexed.len());
        for (shard_index, context_key) in &operation.indexed_context_keys.indexed {
            let context_guard = guards.get(*shard_index);
            let context_shard = context_guard.expect("Missing shard lock");
            let context_value = context_shard.get(context_key);
            context_values.push(context_value);
        }
        let key_guard = guards.get(operation.key_index);
        let key_shard = key_guard.expect("Missing shard lock");
        let key_value = key_shard.get(&operation.key);
        (operation.operator)(key_value, context_values.as_slice(), params)
    }
}

pub struct ParameterizedTransactionBuilder<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    owned_key: fn(&K) -> K,
    custodian: &'txmap Custodian<K, V>,
    prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
    operations: Vec<ParameterizedOperation<K, V, P>>,
}

impl<'txmap, K, V, P> ParameterizedTransactionBuilder<'txmap, K, V, P>
where
    K: Hash + Eq,
{
    pub(crate) fn new(
        indexer: Indexer,
        owned_key: fn(&K) -> K,
        custodian: &'txmap Custodian<K, V>,
    ) -> Self {
        Self {
            indexer,
            owned_key,
            custodian,
            prerequisites: Vec::new(),
            operations: Vec::new(),
        }
    }
    pub fn with_prerequisite<const N: usize, F>(
        mut self,
        name: impl AsRef<str>,
        keys: [K; N],
        prerequisite: F,
    ) -> Self
    where
        F: Fn([Option<&V>; N], &P) -> bool + 'static,
    {
        let prerequisite =
            ParameterizedPrerequisite::new(self.indexer, name.as_ref().into(), keys, prerequisite);
        self.prerequisites.push(prerequisite);
        self
    }
    pub fn with_operation<F>(mut self, key: K, operator: F) -> Self
    where
        F: Fn(Option<&V>, &P) -> Option<V> + 'static,
    {
        let operation = ParameterizedOperation::new(&self.indexer, key, operator);
        self.operations.push(operation);
        self
    }
    pub fn with_operation_and_context<const N: usize, F>(
        mut self,
        key: K,
        operator: F,
        context_keys: [K; N],
    ) -> Self
    where
        F: Fn(Option<&V>, [Option<&V>; N], &P) -> Option<V> + 'static,
    {
        let operation =
            ParameterizedOperation::new_with_context(&self.indexer, key, operator, context_keys);
        self.operations.push(operation);
        self
    }
    pub fn build(self) -> ParameterizedTransaction<'txmap, K, V, P> {
        let Self {
            owned_key,
            custodian,
            prerequisites,
            operations,
            ..
        } = self;
        let mut guards_bitmask: u128 = 0;
        for prerequisite in &prerequisites {
            guards_bitmask |= prerequisite.guards_bitmask;
        }
        for operation in &operations {
            guards_bitmask |= operation.guards_bitmask;
        }
        ParameterizedTransaction {
            owned_key,
            custodian,
            guards_bitmask,
            prerequisites,
            operations,
        }
    }
}
