use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}
