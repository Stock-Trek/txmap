use crate::key::TxKey;
use std::hash::Hash;

pub(crate) struct InsertWithOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &mut STATE) -> V + 'tx>,
}
