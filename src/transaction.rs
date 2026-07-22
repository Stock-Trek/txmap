use crate::{
    custodian::Custodian, finisher::Finisher, finishers::finisher_trait::FinisherTrait,
    guard::Guard, locks::lock_policy::LockPolicy, new_types::BitMask, ops::op_trait::OpTrait,
    result::TxResult,
};
use std::hash::Hash;

pub struct Transaction<'txmap, L, K, V, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub(crate) base: TransactionBase<'txmap, L, K, V, (), F>,
}

impl<'txmap, L, K, V, F> Transaction<'txmap, L, K, V, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    #[must_use]
    pub fn execute(&self) -> TxResult<F::Output> {
        self.base.execute_with_params(&())
    }
}

pub struct ParameterizedTransaction<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub(crate) base: TransactionBase<'txmap, L, K, V, P, F>,
}

impl<'txmap, L, K, V, P, F> ParameterizedTransaction<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    #[must_use]
    pub fn execute(&self, params: &P) -> TxResult<F::Output> {
        self.base.execute_with_params(params)
    }
}

pub(crate) struct TransactionBase<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<L, K, V>,
    pub(crate) guards_bitmask: BitMask,
    pub(crate) guards: Vec<Guard<K, V, P>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<L, K, V, P>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, L, K, V, P, F> TransactionBase<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub fn execute_with_params(&self, params: &P) -> TxResult<F::Output> {
        let mut mutex_guards = self.custodian.guards(self.guards_bitmask);
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met::<L>(&mutex_guards, params) {
                return TxResult::RequirementNotMet(i, guard.name.clone());
            }
        }
        for op in &self.ops {
            op.apply(&mut mutex_guards, params);
        }
        let result = self.finisher.finish::<L>(&mutex_guards);
        TxResult::Completed(result)
    }
}
