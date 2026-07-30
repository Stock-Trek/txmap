use crate::new_types::{HashCode, ShardIndex};
use std::hash::Hash;

/// A key together with its pre-computed hash and shard index.
///
/// Created internally by [`Indexer::indexed_key`]. Used throughout
/// the transaction system to avoid re-hashing.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TxKey<K>
where
    K: Hash + Eq,
{
    /// Pre-computed hash code of the key.
    pub hash_code: HashCode,
    /// The shard this key belongs to.
    pub shard_index: ShardIndex,
    /// The original key value.
    pub key: K,
}
