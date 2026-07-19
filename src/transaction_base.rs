use crate::{
    custodian::Custodian, finisher::Finisher, finishers::finisher_trait::FinisherTrait,
    guard::Guard, ops::op_trait::OpTrait, result::TxResult,
};
use std::hash::Hash;

pub(crate) struct TransactionBase<'txmap, K, V, P, F>
where
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards_bitmask: u128,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, P>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, K, V, P, F> TransactionBase<'txmap, K, V, P, F>
where
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub fn execute_with_params(&self, params: &P) -> TxResult<F::Output> {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met(&mutex_guards) {
                return TxResult::RequirementNotMet(i, guard.name.clone());
            }
        }
        for op in &self.ops {
            op.apply(&mut mutex_guards, params);
        }
        let result = self.finisher.finish(&mutex_guards);
        TxResult::Completed(result)
    }
}
