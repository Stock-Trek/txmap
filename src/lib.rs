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
#[cfg(feature = "rayon")]
pub mod rayon;
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

pub use hasher::DefaultBuildHasher;
pub use immediate::{
    transaction::ImmediateTransaction,
    tx_builder::{ImmediateBuildablePhase, ImmediateBuilderPhase, ImmediateTxBuilder},
};
pub use iter::Iter;
pub use key::TxKey;
pub use lock_policies::{
    lock_policy::LockPolicy, mutex_policy::MutexPolicy, rwlock_policy::RwLockPolicy,
};
pub use new_types::{HashCode, ShardCount, ShardIndex};
pub use prepared::{
    build_phase::{BuildPhase, PreparedBuildablePhase, PreparedBuilderPhase},
    schema::{TxKeySelector, TxKeys, TxSchema},
    transaction::PreparedTransaction,
    tx_builder::PreparedTxBuilder,
};
pub use result::{TryReserveError, TxResult};
pub use shards::Shards;
pub use tx_map::TxMap;
pub use tx_map_builder::TxMapBuilder;

pub mod prelude {
    pub use crate::{
        HashCode, ImmediateBuildablePhase, ImmediateBuilderPhase, ImmediateTransaction,
        ImmediateTxBuilder, LockPolicy, MutexPolicy, PreparedBuildablePhase, PreparedBuilderPhase,
        PreparedTransaction, PreparedTxBuilder, RwLockPolicy, ShardCount, ShardIndex, Shards,
        TryReserveError, TxKey, TxKeySelector, TxKeys, TxMap, TxMapBuilder, TxResult, TxSchema,
        tx_schema,
    };
}
pub use pastey::paste as _paste;
