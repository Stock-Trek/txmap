use crate::{
    custodian::Custodian,
    immediate::{guard::ImmediateGuard, op::ImmediateOp},
    indexer::Indexer,
    new_types::BitMask,
    result::TxResult,
};
use std::hash::{BuildHasher, Hash};

/// An immediate (one-shot) transaction.
///
/// Built via [`ImmediateTxBuilder`](crate::immediate::tx_builder::ImmediateTxBuilder) and executed immediately.
/// Acquires all needed locks, checks guards, applies operations,
/// then releases locks and returns the final state.
pub struct ImmediateTx<'tx, K, V, S, STATE>
where
    S: BuildHasher,
{
    pub(crate) custodian: &'tx Custodian<K, V>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<ImmediateGuard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<ImmediateOp<'tx, K, V, STATE>>,
}

impl<'tx, K, V, S, STATE> ImmediateTx<'tx, K, V, S, STATE>
where
    K: Clone + Hash + Eq,
    S: BuildHasher,
    STATE: Default,
{
    #[must_use]
    /// Consumes self and executes the transaction.
    ///
    /// Acquires locks for all involved shards, verifies
    /// all guard conditions, applies the operations, and returns
    /// the final state wrapped in [`TxResult`].
    pub fn execute(self) -> TxResult<STATE> {
        let Self {
            custodian,
            indexer,
            guards,
            ops,
        } = self;

        let mut total_bitmask = BitMask::ZERO;

        for guard in guards.iter() {
            total_bitmask |= guard.bitmask();
        }
        for op in ops.iter() {
            total_bitmask |= op.bitmask();
        }

        let mut lock_guards = custodian.lock_guard(total_bitmask);
        let mut state = STATE::default();
        for (i, mut guard) in guards.into_iter().enumerate() {
            if !guard.condition_is_met(&mut lock_guards, &mut state) {
                return TxResult::RequirementNotMet {
                    index: i,
                    requirement: guard.name,
                    state,
                };
            }
        }
        for op in ops {
            op.apply::<S>(&mut lock_guards, indexer, &mut state);
        }
        TxResult::Completed { state }
    }
}
