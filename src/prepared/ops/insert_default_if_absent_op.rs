use crate::{
    key::TxKey,
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{ops::op_trait::OpTrait, schema::TxKeySelector},
};
use std::hash::Hash;

pub(crate) struct InsertDefaultIfAbsentOpApply;

impl InsertDefaultIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, KEYS>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
    ) where
        K: Clone + Hash + Eq,
        V: Default,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        lock_guards.insert_if_absent(key, || V::default());
    }
}

pub(crate) struct InsertDefaultIfAbsentOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> OpTrait<K, V, L, KEYS, PARAMS, STATE>
    for InsertDefaultIfAbsentOp<'tx, K, KEYS>
where
    K: Clone + Hash + Eq + 'tx,
    V: Default + 'tx,
    L: LockPolicy + 'tx,
    KEYS: 'tx,
    PARAMS: 'tx,
    STATE: Default + 'tx,
{
    fn read_write_bitmasks(&self, keys: &KEYS) -> (BitMask, BitMask) {
        (
            BitMask::ZERO,
            self.key_selector.get(keys).shard_index.bitmask(),
        )
    }
    fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        _params: &PARAMS,
        _state: &mut STATE,
    ) {
        InsertDefaultIfAbsentOpApply::apply::<K, V, L, KEYS>(&*self.key_selector, lock_guards, keys)
    }
}
