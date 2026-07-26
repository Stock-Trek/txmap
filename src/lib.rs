pub mod custodian;
pub mod indexer;
pub mod key;
pub mod lock_guards;
pub mod lock_policies;
pub mod new_types;
pub mod prepared;
pub mod result;
pub mod shard;
pub mod shards;
#[cfg(test)]
pub mod tests;
pub mod tx_map;

pub mod prelude {
    pub use crate::{
        indexer::Indexer,
        key::TxKey,
        new_types::{BitMask, HashCode, ShardCount, ShardIndex},
        prepared::{
            params::{TxKeySelector, TxKeys, TxSchema, tx_schema},
            transaction::PreparedTransaction,
            tx_builder::{PrepBuildablePhase, PrepBuilderPhase, PrepTxBuilder},
        },
        result::TxResult,
        shards::Shards,
        tx_map::TxMap,
    };
}
pub use pastey::paste as _paste;
