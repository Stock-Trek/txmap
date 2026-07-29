pub mod custodian;
pub mod immediate;
pub mod indexer;
pub mod iter;
pub mod key;
pub mod lock_guards;
pub mod lock_policies;
pub mod new_types;
pub(crate) mod ops;
pub mod prepared;
pub mod result;
#[cfg(feature = "serde")]
pub mod serde;
pub mod shard;
pub mod shards;
#[cfg(test)]
pub mod tests;
pub mod tx_map;

pub mod prelude {
    pub use crate::{
        immediate::{
            transaction::ImmediateTransaction,
            tx_builder::{ImmediateBuildablePhase, ImmediateBuilderPhase, ImmediateTxBuilder},
        },
        indexer::Indexer,
        key::TxKey,
        lock_policies::{
            lock_policy::LockPolicy, mutex_policy::MutexPolicy, rwlock_policy::RwLockPolicy,
        },
        new_types::{BitMask, HashCode, ShardCount, ShardIndex},
        prepared::{
            schema::{TxKeySelector, TxKeys, TxSchema, tx_schema},
            transaction::PreparedTransaction,
            tx_builder::{PreparedBuildablePhase, PreparedBuilderPhase, PreparedTxBuilder},
        },
        result::TxResult,
        shards::Shards,
        tx_map::TxMap,
    };
}
pub use pastey::paste as _paste;
