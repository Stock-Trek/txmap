use crate::new_types::{HashCode, ShardIndex};
use std::hash::Hash;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TxKey<K>
where
    K: Hash + Eq,
{
    pub(crate) hash_code: HashCode,
    pub(crate) shard_index: ShardIndex,
    pub(crate) key: K,
}
