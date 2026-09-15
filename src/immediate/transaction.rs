use crate::{
    custodian::Custodian,
    immediate::{guard::ImmediateGuard, op::ImmediateOp},
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
pub struct ImmediateTx<'tx, K, V, L, S, STATE>
where
    L: LockPolicy,
    S: BuildHasher,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<ImmediateGuard<'tx, K, V, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<ImmediateOp<'tx, K, V, STATE>>,
}

impl<'tx, K, V, L, S, STATE> ImmediateTx<'tx, K, V, L, S, STATE>
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
            mut guards,
            mut ops,
        } = self;

        loop {
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

            // Snapshot the versions observed at route time so we can detect a
            // split or merge that raced with this transaction.
            let mut versions = Vec::with_capacity(guards.len() + ops.len() * 2);
            for guard in guards.iter() {
                versions.push((guard.key.shard_index.0, guard.key.version));
            }
            for op in ops.iter() {
                op.push_versions(&mut versions);
            }

            match custodian.try_lock_guards(
                indexer,
                total_read_bitmask,
                total_write_bitmask,
                &versions,
            ) {
                Some(mut lock_guards) => {
                    let mut state = STATE::default();
                    for (i, mut guard) in std::mem::take(&mut guards).into_iter().enumerate() {
                        if !guard.condition_is_met::<L>(&mut lock_guards, &mut state) {
                            return TxResult::RequirementNotMet {
                                index: i,
                                requirement: guard.name,
                                state,
                            };
                        }
                    }
                    for op in std::mem::take(&mut ops) {
                        op.apply::<L, S>(&mut lock_guards, indexer, &mut state);
                    }
                    return TxResult::Completed { state };
                }
                None => {
                    // Routing changed while we were building the lock set;
                    // refresh every key and try again.
                    for guard in guards.iter_mut() {
                        guard.reroute(custodian);
                    }
                    for op in ops.iter_mut() {
                        op.reroute(custodian);
                    }
                }
            }
        }
    }
}
