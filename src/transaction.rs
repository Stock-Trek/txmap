use crate::{
    custodian::Custodian, guard::Guard, map_op::MapOp, mut_op::MutOp, op::Op, result::TxResult,
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct Transaction<'txmap, K, V, R>
where
    K: Clone + Hash + Eq,
{
    pub(crate) owned_key: fn(&K) -> K,
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards_bitmask: u128,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Op<K, V>>,
    pub(crate) get: Box<dyn Fn(&V) -> R>,
}

impl<'txmap, K, V, R> Transaction<'txmap, K, V, R>
where
    K: Clone + Hash + Eq,
{
    pub fn execute(&self) -> TxResult {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, guard) in self.guards.iter().enumerate() {
            if !self.is_condition_met(guard, &mutex_guards) {
                return TxResult::ConditionNotMet(i, guard.name.clone());
            }
        }
        for op in &self.ops {
            match op {
                Op::Map(map_op) => {
                    let new_value = self.mapped_value(map_op, &mutex_guards);
                    let guard = mutex_guards.get_mut(map_op.key_index);
                    let shard = guard.expect("Missing shard lock");
                    match new_value {
                        Some(v) => shard.insert((self.owned_key)(&map_op.key), v),
                        None => shard.remove(&map_op.key),
                    };
                }
                Op::Mut(mut_op) => {
                    self.mut_value(mut_op, &mut mutex_guards);
                }
            }
        }
        TxResult::Completed
    }

    fn is_condition_met(
        &self,
        guard: &Guard<K, V>,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> bool {
        let mut values = Vec::with_capacity(guard.indexed_keys.indexed.len());
        for (shard_index, key) in &guard.indexed_keys.indexed {
            let mutex_guard = mutex_guards.get(*shard_index);
            let shard = mutex_guard.expect("Missing shard lock");
            let value = shard.get(key);
            values.push(value);
            if !(guard.is_condition_met)(&values) {
                return false;
            }
        }
        true
    }
    fn mapped_value(
        &self,
        map_op: &MapOp<K, V>,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> Option<V> {
        let mut context_values = Vec::with_capacity(map_op.indexed_context_keys.indexed.len());
        for (shard_index, context_key) in &map_op.indexed_context_keys.indexed {
            let context_guard = mutex_guards.get(*shard_index);
            let context_shard = context_guard.expect("Missing shard lock");
            let context_value = context_shard.get(context_key);
            context_values.push(context_value);
        }
        let key_guard = mutex_guards.get(map_op.key_index);
        let key_shard = key_guard.expect("Missing shard lock");
        let key_value = key_shard.get(&map_op.key);
        (map_op.mapper)(key_value, context_values.as_slice())
    }
    fn mut_value(
        &self,
        mut_op: &MutOp<K, V>,
        mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) {
        let key_guard = mutex_guards.get_mut(mut_op.key_index);
        let key_shard = key_guard.expect("Missing shard lock");
        let key_value = key_shard.get_mut(&mut_op.key);
        if let Some(mut mutable_value) = key_value {
            (mut_op.mutator)(&mut mutable_value)
        } else if let Some(value_generator) = &mut_op.value_generator {
            let new_value = value_generator();
            key_shard.insert((self.owned_key)(&mut_op.key), new_value);
        }
    }
}
