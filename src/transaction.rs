use crate::{
    custodian::Custodian, finisher::Finisher, finishers::finisher_trait::FinisherTrait,
    guard::Guard, op::Op, result::TxResult,
};
use std::hash::Hash;

pub struct Transaction<'txmap, K, V, F>
where
    K: Clone + Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards_bitmask: u128,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Op<K, V>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, K, V, F> Transaction<'txmap, K, V, F>
where
    K: Clone + Hash + Eq,
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
            op.apply(&mut mutex_guards);
        }
        let result = self.finisher.finish(&mutex_guards);
        TxResult::Completed(result)
    }
}
