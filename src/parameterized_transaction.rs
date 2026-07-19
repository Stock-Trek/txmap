use crate::{
    finishers::finisher_trait::FinisherTrait, result::TxResult, transaction_base::TransactionBase,
};
use std::hash::Hash;

pub struct ParameterizedTransaction<'txmap, K, V, P, F>
where
    F: FinisherTrait<K, V>,
{
    pub(crate) base: TransactionBase<'txmap, K, V, P, F>,
}

impl<'txmap, K, V, P, F> ParameterizedTransaction<'txmap, K, V, P, F>
where
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub fn execute(&self, params: &P) -> TxResult<F::Output> {
        self.base.execute_with_params(params)
    }
}
