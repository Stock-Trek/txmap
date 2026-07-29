use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct SwapValueOp<K>
where
    K: Hash + Eq,
{
    pub key_a: TxKey<K>,
    pub key_b: TxKey<K>,
}
