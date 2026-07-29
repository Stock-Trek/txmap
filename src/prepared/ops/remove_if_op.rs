use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct RemoveIfOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, &V, &PARAMS, &mut STATE) -> bool + 'tx>,
}
