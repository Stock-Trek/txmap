use crate::{
    custodian::Custodian,
    indexer::Indexer,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{guard::Guard, op::PreparedOp, schema::TxKeys},
    result::TxResult,
};
use std::hash::{BuildHasher, Hash};

/// A prepared (re-usable) transaction.
///
/// Built via [`PreparedTxBuilder`] and executed multiple times with
/// different keys and parameters. The transaction plan (which shards
/// to lock and which operations to apply) is determined at build time.
pub struct PreparedTransaction<'tx, K, V, L, S, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
    STATE: Default,
{
    pub(crate) custodian: &'tx Custodian<K, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    pub(crate) guards: Vec<Guard<'tx, K, V, KEYS, PARAMS, STATE>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<PreparedOp<'tx, K, V, KEYS, PARAMS, STATE>>,
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE> PreparedTransaction<'tx, K, V, L, S, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq,
    L: LockPolicy,
    S: BuildHasher,
    STATE: Default,
{
    #[must_use]
    /// Executes the transaction with the given keys and parameters.
    ///
    /// The keys are hashed, locks are acquired, guards are checked,
    /// and operations are applied. Returns the final state wrapped in
    /// [`TxResult`].
    pub fn execute<RAW>(&self, keys: RAW, params: PARAMS) -> TxResult<STATE>
    where
        RAW: TxKeys<K, KEYS, S>,
    {
        let keys = keys.into_indexed(self.custodian.shard_count, self.indexer);
        let mut total_read_bitmask = BitMask::ZERO;
        let mut total_write_bitmask = BitMask::ZERO;

        // get all bitmasks
        for guard in self.guards.iter() {
            total_read_bitmask |= guard.read_bitmask(&keys);
        }
        for op in self.ops.iter() {
            let (read_bitmask, write_bitmask) = op.read_write_bitmasks(&keys);
            total_read_bitmask |= read_bitmask;
            total_write_bitmask |= write_bitmask;
        }
        // ensure locks are either read or write, not both
        total_read_bitmask &= !total_write_bitmask;

        let mut lock_guards = self
            .custodian
            .lock_guards(total_read_bitmask, total_write_bitmask);
        let mut state = STATE::default();
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met::<L>(&mut lock_guards, &keys, &params, &mut state) {
                return TxResult::RequirementNotMet(i, guard.name.clone(), state);
            }
        }
        for op in self.ops.iter() {
            op.apply::<L, S>(&mut lock_guards, &keys, &params, self.indexer, &mut state);
        }
        TxResult::Completed(state)
    }
}
