use crate::{
    custodian::Custodian,
    hasher::DefaultBuildHasher,
    indexer::Indexer,
    lock_policies::{lock_policy::LockPolicy, mutex_policy::MutexPolicy},
    new_types::BitMask,
    prepared::{guard::Guard, op::PreparedOp, schema::TxKeys, schema::TxSchema},
    result::TxResult,
};
use std::hash::{BuildHasher, Hash};

/// A prepared (re-usable) transaction.
///
/// Built via [`PreparedTxBuilder`](crate::prepared::tx_builder::PreparedTxBuilder) and executed multiple times with
/// different keys and parameters. The transaction plan (which shards
/// to lock and which operations to apply) is determined at build time.
///
/// The only generics that must be written are the transaction's [`TxSchema`]
/// and the map's value type; the lock policy and hasher default to
/// `MutexPolicy` and `DefaultBuildHasher`. The keys, params and state types
/// are associated types of the schema, making the transaction easy to store
/// in structs, collections, or behind an `Arc`:
///
/// ```
/// use txmap::prelude::*;
///
/// tx_schema! {
///     Demo,
///     keys: [key],
///     params: {},
///     state: {},
/// }
///
/// struct App<'tx> {
///     demo: PreparedTransaction<'tx, Demo<String>, u64>,
/// }
/// ```
pub struct PreparedTransaction<'tx, SCHEMA, V, L = MutexPolicy, S = DefaultBuildHasher>
where
    SCHEMA: TxSchema + 'tx,
    V: 'tx,
    L: LockPolicy,
    S: BuildHasher,
{
    pub(crate) custodian: &'tx Custodian<SCHEMA::Key, V, L>,
    pub(crate) indexer: &'tx Indexer<S>,
    #[allow(clippy::type_complexity)]
    pub(crate) guards:
        Vec<Guard<'tx, SCHEMA::Key, V, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>>,
    #[allow(clippy::type_complexity)]
    pub(crate) ops:
        Vec<PreparedOp<'tx, SCHEMA::Key, V, SCHEMA::IndexedKeys, SCHEMA::Params, SCHEMA::State>>,
}

impl<'tx, SCHEMA, V, L, S> PreparedTransaction<'tx, SCHEMA, V, L, S>
where
    SCHEMA: TxSchema + 'tx,
    V: 'tx,
    L: LockPolicy,
    S: BuildHasher,
    SCHEMA::Key: Clone + Hash + Eq,
    SCHEMA::Keys: TxKeys<SCHEMA::Key, SCHEMA::IndexedKeys, S>,
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
            if !guard.is_condition_met::<L>(&mut lock_guards, &keys, &params, &mut state) {
                return TxResult::RequirementNotMet {
                    index: i,
                    requirement: guard.name.clone(),
                    state,
                };
            }
        }
        for op in self.ops.iter() {
            op.apply::<L, S>(
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
