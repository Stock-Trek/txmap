use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct InsertDefaultOp<K>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
}
