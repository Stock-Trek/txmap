use crate::{
    builder_traits::IntoTransaction,
    custodian::Custodian,
    guard::Guard,
    op::Op,
    result::{MISSING_MUTEX_GUARD_ERROR, TxResult},
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

impl<'txmap, K, V, R> IntoTransaction<'txmap, K, V, R> for Transaction<'txmap, K, V, R>
where
    K: Clone + Hash + Eq,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, R> {
        self
    }
}

impl<'txmap, K, V, R> Transaction<'txmap, K, V, R>
where
    K: Clone + Hash + Eq,
    V: Default,
{
    pub fn execute(&self) -> TxResult {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, guard) in self.guards.iter().enumerate() {
            if !self.is_condition_met(guard, &mutex_guards) {
                return TxResult::ConditionNotMet(i, guard.name.clone());
            }
        }
        for op in &self.ops {
            execute_op(op, &mut mutex_guards);
        }
        TxResult::Completed
    }
}

impl<'txmap, K, V, R> Transaction<'txmap, K, V, R>
where
    K: Clone + Hash + Eq,
{
    fn is_condition_met(
        &self,
        guard: &Guard<K, V>,
        mutex_guards: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>,
    ) -> bool {
        let mut values = Vec::with_capacity(guard.indexed_keys.indexed.len());
        for (shard_index, key) in &guard.indexed_keys.indexed {
            let mutex_guard = mutex_guards.get(*shard_index);
            let shard = mutex_guard.expect(MISSING_MUTEX_GUARD_ERROR);
            let value = shard.get(key);
            values.push(value);
            if !(guard.is_condition_met)(&values) {
                return false;
            }
        }
        true
    }
}

fn execute_op<K, V>(op: &Op<K, V>, mutex_guards: &mut IntMap<u8, MutexGuard<'_, HashMap<K, V>>>)
where
    K: Clone + Hash + Eq,
    V: Default,
{
    use crate::ops::op_trait::OpTrait;
    match op {
        Op::InsertWith(op) => op.apply(mutex_guards),
        Op::InsertDefault(op) => op.apply(mutex_guards),
        Op::Modify(op) => op.apply(mutex_guards),
        Op::ModifyPeek(op) => op.apply(mutex_guards),
        Op::ModifyOrInsertWith(op) => op.apply(mutex_guards),
        Op::ModifyPeekOrInsertWith(op) => op.apply(mutex_guards),
        Op::ModifyOrDefault(op) => op.apply(mutex_guards),
        Op::ModifyPeekOrDefault(op) => op.apply(mutex_guards),
        Op::Map(op) => op.apply(mutex_guards),
        Op::MapPeek(op) => op.apply(mutex_guards),
        Op::Mut(op) => op.apply(mutex_guards),
        Op::SwapValue(op) => op.apply(mutex_guards),
        Op::MoveValue(op) => op.apply(mutex_guards),
        Op::Remove(op) => op.apply(mutex_guards),
        Op::RemoveIf(op) => op.apply(mutex_guards),
        Op::Retain(op) => op.apply(mutex_guards),
        Op::RetainIf(op) => op.apply(mutex_guards),
        Op::Clear(op) => op.apply(mutex_guards),
        Op::RemoveAnyIf(op) => op.apply(mutex_guards),
        Op::RetainAnyIf(op) => op.apply(mutex_guards),
    }
}
