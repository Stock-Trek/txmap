use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct ModifyOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub mutate: Box<dyn Fn(&K, &mut V, &mut STATE) + 'tx>,
}
