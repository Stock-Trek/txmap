use crate::{
    custodian::Custodian, operation::Operation, prerequisite::Prerequisite, result::TxResult,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct Transaction<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards_bitmask: u128,
    pub(crate) prerequisites: Vec<Prerequisite<K, V>>,
    pub(crate) operations: Vec<Operation<K, V>>,
}

impl<'txmap, K, V> Transaction<'txmap, K, V>
where
    K: Hash + Eq,
{
    pub fn execute(&self) -> TxResult {
        let mut guards = self.custodian.guards(self.guards_bitmask);
        for (i, prerequisite) in self.prerequisites.iter().enumerate() {
            if !self.is_prerequisite_met(prerequisite, &guards) {
                return TxResult::UnmetPrerequisite(i, prerequisite.name.clone());
            }
        }
        for operation in &self.operations {
            let new_value = self.operation_value(operation, &guards);
            let guard = guards.get_mut(operation.key_index);
            let shard = guard.expect("Missing shard lock");
            match new_value {
                Some(v) => shard.insert((self.owned_key)(&operation.key), v),
                None => shard.remove(&operation.key),
            };
        }
        TxResult::Completed
    }

    fn is_prerequisite_met(
        &self,
        prerequisite: &Prerequisite<K, V>,
        guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> bool {
        let mut values = Vec::with_capacity(prerequisite.indexed_keys.indexed.len());
        for (shard_index, key) in &prerequisite.indexed_keys.indexed {
            let guard = guards.get(*shard_index);
            let shard = guard.expect("Missing shard lock");
            let value = shard.get(key);
            values.push(value);
            if !(prerequisite.is_satisfied)(&values) {
                return false;
            }
        }
        true
    }
    fn operation_value(
        &self,
        operation: &Operation<K, V>,
        guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
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
        (operation.operator)(key_value, context_values.as_slice())
    }
}
