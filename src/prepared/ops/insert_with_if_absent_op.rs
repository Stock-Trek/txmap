use crate::{
    key::TxKey,
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{ops::op_trait::OpTrait, schema::TxKeySelector},
};
use std::hash::Hash;

pub(crate) struct InsertWithIfAbsentOpApply;

impl InsertWithIfAbsentOpApply {
    pub(crate) fn apply<K, V, L, KEYS, PARAMS, STATE>(
        key_selector: &dyn TxKeySelector<TxKey<K>, KEYS>,
        value_generator: &dyn Fn(&K, &PARAMS, &mut STATE) -> V,
        lock_guards: &mut LockGuards<'_, K, V, L>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) where
        K: Clone + Hash + Eq,
        L: LockPolicy,
    {
        let key = key_selector.get(keys);
        lock_guards.insert_if_absent(key, || (value_generator)(&key.key, params, state));
    }
}

pub(crate) struct InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
    STATE: Default,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub value_generator: Box<dyn Fn(&K, &PARAMS, &mut STATE) -> V + 'tx>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> OpTrait<K, V, L, KEYS, PARAMS, STATE>
    for InsertWithIfAbsentOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
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
        params: &PARAMS,
        state: &mut STATE,
    ) {
        InsertWithIfAbsentOpApply::apply(
            &*self.key_selector,
            &*self.value_generator,
            lock_guards,
            keys,
            params,
            state,
        )
    }
}
