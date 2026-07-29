use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct ModifyOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub mutate: Box<dyn Fn(&K, &mut V, &PARAMS, &mut STATE) + 'tx>,
}
