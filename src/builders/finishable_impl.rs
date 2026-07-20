use crate::{
    builders::builder_traits::IntoTransaction,
    custodian::Custodian,
    finisher::Finisher,
    finishers::finisher_trait::FinisherTrait,
    guard::Guard,
    ops::op_trait::OpTrait,
    transaction::{Transaction, TransactionBase},
};

pub struct TxFinishableImpl<'txmap, K, V, F>
where
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<K, V>,
    pub(crate) guards: Vec<Guard<K, V>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<K, V, ()>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, K, V, F> IntoTransaction<'txmap, K, V, F> for TxFinishableImpl<'txmap, K, V, F>
where
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> Transaction<'txmap, K, V, F> {
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
        guards_bitmask |= finisher.guards_bitmask();
        let base = TransactionBase {
            custodian,
            guards_bitmask,
            guards,
            ops,
            finisher,
        };
        Transaction { base }
    }
}
