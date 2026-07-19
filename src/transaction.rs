use crate::{
    finishers::finisher_trait::FinisherTrait, result::TxResult, transaction_base::TransactionBase,
};
use std::hash::Hash;

pub struct Transaction<'txmap, K, V, F>
where
    F: FinisherTrait<K, V>,
{
    pub(crate) base: TransactionBase<'txmap, K, V, (), F>,
}

impl<'txmap, K, V, F> Transaction<'txmap, K, V, F>
where
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub fn execute(&self) -> TxResult<F::Output> {
        self.base.execute_with_params(&())
    }
}
