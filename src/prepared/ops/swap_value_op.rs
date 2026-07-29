use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct SwapValueOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector_a: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    pub key_selector_b: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}
