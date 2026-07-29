use crate::{key::TxKey, prepared::schema::TxKeySelector};
use std::hash::Hash;

pub(crate) struct GetOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub get: Box<dyn Fn(&K, Option<&V>, &PARAMS, &mut STATE) + 'tx>,
}
