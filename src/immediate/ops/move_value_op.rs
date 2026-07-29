use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct MoveValueOp<K>
where
    K: Hash + Eq,
{
    pub key_from: TxKey<K>,
    pub key_to: TxKey<K>,
}
