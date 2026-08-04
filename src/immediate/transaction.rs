use crate::{
    custodian::Custodian,
    immediate::{guard::Guard, op::ImmediateOp},
    indexer::Indexer,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    result::TxResult,
};
use std::hash::{BuildHasher, Hash};

/// An immediate (one-shot) transaction.
///
/// Built via [`ImmediateTxBuilder`](crate::immediate::tx_builder::ImmediateTxBuilder) and executed immediately.
/// Acquires all needed locks, checks guards, applies operations,
/// then releases locks and returns the final state.
pub struct ImmediateTransaction<'tx, K, V, L, S, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
    STATE: Default,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<Guard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<ImmediateOp<'tx, K, V, STATE>>,
}

impl<'tx, K, V, L, S, STATE> ImmediateTransaction<'tx, K, V, L, S, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
    STATE: Default,
{
    #[must_use]
    /// Consumes self and executes the transaction.
    ///
    /// Acquires read/write locks for all involved shards, verifies
    /// all guard conditions, applies the operations, and returns
    /// the final state wrapped in [`TxResult`].
    pub fn execute(self) -> TxResult<STATE> {
        let Self {
            custodian,
            indexer,
            guards,
            ops,
        } = self;

        let mut total_read_bitmask = BitMask::ZERO;
        let mut total_write_bitmask = BitMask::ZERO;

        // get all bitmasks
        for guard in guards.iter() {
            total_read_bitmask |= guard.read_bitmask();
        }
        for op in ops.iter() {
            let (read_bitmask, write_bitmask) = op.read_write_bitmasks();
            total_read_bitmask |= read_bitmask;
            total_write_bitmask |= write_bitmask;
        }
        // ensure locks are either read or write, not both
        total_read_bitmask &= !total_write_bitmask;

        let mut lock_guards = custodian.lock_guards(total_read_bitmask, total_write_bitmask);
        let mut state = STATE::default();
        for (i, guard) in guards.into_iter().enumerate() {
            let guard_name = guard.name.clone();
            if !guard.condition_is_met::<L>(&mut lock_guards, &mut state) {
                return TxResult::RequirementNotMet(i, guard_name, state);
            }
        }
        for op in ops {
            op.apply::<L, S>(&mut lock_guards, indexer, &mut state);
        }
        TxResult::Completed(state)
    }
}
