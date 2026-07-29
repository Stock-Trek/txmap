use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct MoveValueOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector_from: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    pub key_selector_to: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}
