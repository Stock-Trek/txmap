use crate::{prepared::schema::TxSchema, result::TxResult};

/// A prepared (re-usable) transaction.
///
/// Built via [`PreparedTxBuilder`](crate::prepared::tx_builder::PreparedTxBuilder) and executed multiple times with
/// different keys and parameters. The transaction plan (which shards
/// to lock and which operations to apply) is determined at build time.
///
/// The only generic parameter is the transaction's [`TxSchema`], making it
/// easy to store in structs, collections, or behind an `Arc`:
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
///     demo: PreparedTransaction<'tx, Demo<String>>,
/// }
/// ```
pub struct PreparedTransaction<'tx, SCHEMA>
where
    SCHEMA: TxSchema,
{
    #[allow(clippy::type_complexity)]
    pub(crate) exec: Box<dyn Fn(SCHEMA::Keys, SCHEMA::Params) -> TxResult<SCHEMA::State> + 'tx>,
}

impl<'tx, SCHEMA> PreparedTransaction<'tx, SCHEMA>
where
    SCHEMA: TxSchema,
{
    #[must_use]
    /// Executes the transaction with the given keys and parameters.
    ///
    /// The keys are hashed, locks are acquired, guards are checked,
    /// and operations are applied. Returns the final state wrapped in
    /// [`TxResult`].
    pub fn execute(&self, keys: SCHEMA::Keys, params: SCHEMA::Params) -> TxResult<SCHEMA::State> {
        (self.exec)(keys, params)
    }
}
