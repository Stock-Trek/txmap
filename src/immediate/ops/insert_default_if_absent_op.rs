use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOp<K>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
}
