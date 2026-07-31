//! A concurrent transactional hash map with fine-grained locking, internal mutability
//! and composable transactions.

pub mod custodian;
pub mod hasher;
pub mod immediate;
pub mod indexer;
pub mod iter;
pub mod key;
pub mod lock_guards;
pub mod lock_policies;
pub mod multi_shard_ops;
pub mod new_types;
pub mod prepared;
pub mod result;
#[cfg(feature = "serde")]
pub mod serde;
pub mod shard;
pub mod shard_ops;
pub mod shards;
#[cfg(test)]
pub mod tests;
pub mod tx_map;
pub mod tx_map_builder;

// The library's primary types are re-exported at the crate root so they can
// be imported directly (e.g. `use txmap::TxMap;`) and are visible on the
// crate's top-level docs page. The full module tree stays public for
// advanced users who need to reach into specific implementations.
pub use immediate::{
    transaction::ImmediateTransaction,
    tx_builder::{ImmediateBuildablePhase, ImmediateBuilderPhase, ImmediateTxBuilder},
};
pub use hasher::DefaultBuildHasher;
pub use iter::Iter;
pub use key::TxKey;
pub use lock_policies::{
    lock_policy::LockPolicy, mutex_policy::MutexPolicy, rwlock_policy::RwLockPolicy,
};
pub use new_types::{HashCode, ShardCount, ShardIndex};
pub use prepared::{
    schema::{TxKeySelector, TxKeys, TxSchema, tx_schema},
    transaction::PreparedTransaction,
    tx_builder::{PreparedBuildablePhase, PreparedBuilderPhase, PreparedTxBuilder},
};
pub use result::TxResult;
pub use shards::Shards;
pub use tx_map::TxMap;
pub use tx_map_builder::TxMapBuilder;

/// Prelude module that re-exports the most commonly used types.
///
/// Bring the common API into scope with a glob import:
/// `use txmap::prelude::*;`
pub mod prelude {
    pub use crate::{
        HashCode, ImmediateBuildablePhase, ImmediateBuilderPhase, ImmediateTransaction,
        ImmediateTxBuilder, LockPolicy, MutexPolicy, PreparedBuildablePhase, PreparedBuilderPhase,
        PreparedTransaction, PreparedTxBuilder, RwLockPolicy, ShardCount, ShardIndex, Shards,
        TxKey, TxKeySelector, TxKeys, TxMap, TxMapBuilder, TxResult, TxSchema, tx_schema,
    };
}
pub use pastey::paste as _paste;
