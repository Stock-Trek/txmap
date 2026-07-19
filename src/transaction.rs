use crate::{
    custodian::Custodian,
    finisher::Finisher,
    finishers::finisher_trait::FinisherTrait,
    guard::Guard,
    ops::op_trait::ParameterizedOpTrait,
    parameterized_prerequisite::ParameterizedPrerequisite,
    result::{MISSING_MUTEX_GUARD_ERROR, TxResult},
};
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct Transaction<'txmap, K, V, P = (), F = crate::finishers::none_finisher::NoneFinisher>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards_bitmask: u128,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) param_prerequisites: Vec<ParameterizedPrerequisite<K, V, P>>,
    pub(crate) ops: Vec<Box<dyn ParameterizedOpTrait<K, V, P>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, K, V, F> Transaction<'txmap, K, V, (), F>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    F: FinisherTrait<K, V>,
{
    pub fn execute(&self) -> TxResult<F::Output> {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met(&mutex_guards) {
                return TxResult::ConditionNotMet(i, guard.name.clone());
            }
        }
        for op in &self.ops {
            op.apply(&mut mutex_guards, &());
        }
        let result = self.finisher.finish(&mutex_guards);
        TxResult::Completed(result)
    }
}

impl<'txmap, K, V, P, F> Transaction<'txmap, K, V, P, F>
where
    K: Clone + Hash + Eq + 'static,
    V: 'static,
    P: 'static,
    F: FinisherTrait<K, V>,
{
    pub fn execute_params(&self, params: &P) -> TxResult<F::Output> {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, prerequisite) in self.param_prerequisites.iter().enumerate() {
            if !self.is_prerequisite_met(prerequisite, &mutex_guards, params) {
                return TxResult::ConditionNotMet(i, prerequisite.name.clone());
            }
        }
        for op in &self.ops {
            op.apply(&mut mutex_guards, params);
        }
        let result = self.finisher.finish(&mutex_guards);
        TxResult::Completed(result)
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
            let shard = guard.expect(MISSING_MUTEX_GUARD_ERROR);
            let value = shard.get(key);
            values.push(value);
            if !(prerequisite.is_satisfied)(&values, params) {
                return false;
            }
        }
        true
    }
}
