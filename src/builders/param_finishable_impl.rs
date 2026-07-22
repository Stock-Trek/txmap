use crate::{
    builders::builder_traits::IntoParamTransaction,
    custodian::Custodian,
    finisher::Finisher,
    finishers::finisher_trait::FinisherTrait,
    guard::Guard,
    locks::lock_policy::LockPolicy,
    new_types::BitMask,
    ops::op_trait::OpTrait,
    transaction::{ParameterizedTransaction, TransactionBase},
};
use std::hash::Hash;

pub struct TxParamFinishableImpl<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    pub(crate) custodian: &'txmap Custodian<L, K, V>,
    pub(crate) guards: Vec<Guard<K, V, P>>,
    pub(crate) ops: Vec<Box<dyn OpTrait<L, K, V, P>>>,
    pub(crate) finisher: Finisher<K, V, F>,
}

impl<'txmap, L, K, V, P, F> IntoParamTransaction<'txmap, L, K, V, P, F>
    for TxParamFinishableImpl<'txmap, L, K, V, P, F>
where
    L: LockPolicy,
    K: Hash + Eq,
    F: FinisherTrait<K, V>,
{
    fn into_transaction(self) -> ParameterizedTransaction<'txmap, L, K, V, P, F> {
        let Self {
            custodian,
            guards,
            ops,
            finisher,
            ..
        } = self;
        let mut guards_bitmask = BitMask::default();
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
        ParameterizedTransaction { base }
    }
}
