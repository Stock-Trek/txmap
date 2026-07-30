use crate::new_types::{HashCode, ShardIndex};
use std::hash::Hash;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TxKey<K>
where
    K: Hash + Eq,
{
    pub hash_code: HashCode,
    pub shard_index: ShardIndex,
    pub key: K,
}
