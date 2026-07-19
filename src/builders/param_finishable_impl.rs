use crate::{
    builders::builder_traits::IntoParamTransaction,
    custodian::Custodian,
    finisher::Finisher,
    finishers::finisher_trait::FinisherTrait,
    guard::Guard,
    ops::op_trait::OpTrait,
    transaction::{ParameterizedTransaction, TransactionBase},
};

pub struct TxParamFinishableImpl<'txmap, K, V, P, F>
where
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V, P>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, P>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, K, V, P, F> IntoParamTransaction<'txmap, K, V, P, F>
    for TxParamFinishableImpl<'txmap, K, V, P, F>
where
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, K, V, P, F> {
        let Self {
            custodian,
            guards,
            ops,
            finisher,
            ..
        } = self;
        let mut guards_bitmask: u128 = 0;
        for guard in &guards {
            guards_bitmask |= guard.guards_bitmask;
        }
        for op in &ops {
            guards_bitmask |= op.guards_bitmask();
        }
        let base = TransactionBase {
            custodian,
            guards_bitmask,
            guards,
            ops,
            finisher,
        };
        ParameterizedTransaction { base }
    }
}
