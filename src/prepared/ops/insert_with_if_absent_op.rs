use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &PARAMS, &mut STATE) -> V + 'tx>,
}
