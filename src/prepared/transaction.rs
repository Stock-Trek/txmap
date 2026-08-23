use crate::{
    custodian::Custodian,
    indexer::Indexer,
    new_types::BitMask,
    prepared::{guard::Guard, op::PreparedOp, schema::TxKeys, schema::TxSchema},
    result::TxResult,
};
use std::hash::Hash;

/// A prepared (re-usable) transaction.
///
/// Built via [`PreparedTxBuilder`](crate::prepared::tx_builder::PreparedTxBuilder) and executed multiple times with
/// different keys and parameters. The transaction plan (which shards
/// to lock and which operations to apply) is determined at build time.
///
/// The only generic parameter is the transaction's [`TxSchema`], which
/// carries the key, value, keys, params, state, lock policy and hasher as
/// associated types, making it easy to store in structs, collections, or
/// behind an `Arc`:
///
/// ```
/// use txmap::prelude::*;
///
/// tx_schema! {
///     Demo,
///     keys: [key],
///     params: {},
///     state: {},
///     value: u64,
/// }
///
/// struct App<'tx> {
///     demo: PreparedTransaction<'tx, Demo<String>>,
/// }
/// ```
pub struct PreparedTransaction<'tx, SCHEMA>
where
    SCHEMA: TxSchema + 'tx,
{
    pub(crate) custodian: &'tx Custodian<SCHEMA::Key, SCHEMA::Value, SCHEMA::LockPolicy>,
    pub(crate) indexer: &'tx Indexer<SCHEMA::Hasher>,
    #[allow(clippy::type_complexity)]
    pub(crate) guards: Vec<
        Guard<'tx, SCHEMA::Key, SCHEMA::Value, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>,
    >,
    #[allow(clippy::type_complexity)]
    pub(crate) ops: Vec<
        PreparedOp<
            'tx,
            SCHEMA::Key,
            SCHEMA::Value,
            SCHEMA::IndexedKeys,
            SCHEMA::Params,
            SCHEMA::State,
        >,
    >,
}

impl<'tx, SCHEMA> PreparedTransaction<'tx, SCHEMA>
where
    SCHEMA: TxSchema + 'tx,
    SCHEMA::Key: Clone + Hash + Eq,
{
    #[must_use]
    /// Executes the transaction with the given keys and parameters.
    ///
    /// The keys are hashed, locks are acquired, guards are checked,
    /// and operations are applied. Returns the final state wrapped in
    /// [`TxResult`].
    pub fn execute(&self, keys: SCHEMA::Keys, params: SCHEMA::Params) -> TxResult<SCHEMA::State> {
        let mut keys = keys.into_indexed(self.custodian.shard_count, self.indexer);
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
        let mut state = SCHEMA::State::default();
        for (i, guard) in self.guards.iter().enumerate() {
            if !guard.is_condition_met::<SCHEMA::LockPolicy>(
                &mut lock_guards,
                &keys,
                &params,
                &mut state,
            ) {
                return TxResult::RequirementNotMet {
                    index: i,
                    requirement: guard.name.clone(),
                    state,
                };
            }
        }
        for op in self.ops.iter() {
            op.apply::<SCHEMA::LockPolicy, SCHEMA::Hasher>(
                &mut lock_guards,
                &mut keys,
                &params,
                self.indexer,
                &mut state,
            );
        }
        TxResult::Completed { state }
    }
}
