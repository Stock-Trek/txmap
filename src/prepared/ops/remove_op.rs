use crate::{
    key::TxKey,
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{ops::op_trait::OpTrait, schema::TxKeySelector},
};
use std::hash::{BuildHasher, Hash};

pub(crate) struct RemoveOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq,
{
    pub key_selector: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    #[allow(clippy::type_complexity)]
    pub on_remove: Box<dyn Fn(Option<(K, V)>, &PARAMS, &mut STATE) + 'tx>,
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE> OpTrait<K, V, L, S, KEYS, PARAMS, STATE>
    for RemoveOp<'tx, K, V, KEYS, PARAMS, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
    S: BuildHasher + 'tx,
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
        lock_guards: &mut LockGuards<'_, K, V, L, S>,
        keys: &KEYS,
        params: &PARAMS,
        state: &mut STATE,
    ) {
        let key = self.key_selector.get(keys);
        let removed_entry = lock_guards.remove_entry(key);
        (self.on_remove)(removed_entry, params, state)
    }
}
