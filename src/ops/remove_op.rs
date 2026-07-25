use crate::{
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    ops::op_trait::OpTrait,
    params::{TxKey, TxKeySelector},
};
use std::hash::Hash;

pub(crate) struct RemoveOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}

impl<'tx, K, V, L, KEYS, PARAMS, STATE> OpTrait<K, V, L, KEYS, PARAMS, STATE>
    for RemoveOp<'tx, K, KEYS>
where
    K: Hash + Eq + 'tx,
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
        _: &PARAMS,
        _: &mut STATE,
    ) {
        let key = self.key_selector.get(keys);
        lock_guards.remove_entry(key);
    }
}
