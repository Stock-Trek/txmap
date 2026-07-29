use crate::{
    key::TxKey,
    lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy,
    new_types::BitMask,
    prepared::{ops::op_trait::OpTrait, schema::TxKeySelector},
};
use std::hash::{BuildHasher, Hash};

pub(crate) struct SwapValueOp<'tx, K, KEYS>
where
    K: Hash + Eq,
{
    pub key_selector_a: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
    pub key_selector_b: Box<dyn TxKeySelector<TxKey<K>, KEYS> + 'tx>,
}

impl<'tx, K, V, L, S, KEYS, PARAMS, STATE> OpTrait<K, V, L, S, KEYS, PARAMS, STATE>
    for SwapValueOp<'tx, K, KEYS>
where
    K: Clone + Hash + Eq + 'tx,
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
            self.key_selector_a.get(keys).shard_index.bitmask()
                | self.key_selector_b.get(keys).shard_index.bitmask(),
        )
    }
    fn apply(
        &self,
        lock_guards: &mut LockGuards<'_, K, V, L, S>,
        keys: &KEYS,
        _params: &PARAMS,
        _state: &mut STATE,
    ) {
        let key_a = self.key_selector_a.get(keys);
        let key_b = self.key_selector_b.get(keys);
        let a = lock_guards.remove_entry(key_a);
        let b = lock_guards.remove_entry(key_b);
        match a {
            Some((a_key, a_value)) => match b {
                Some((b_key, b_value)) => {
                    lock_guards.insert_with_duplicate_key(key_a, a_key, b_value);
                    lock_guards.insert_with_duplicate_key(key_b, b_key, a_value);
                }
                None => {
                    lock_guards.insert(key_b, a_value);
                }
            },
            None => {
                if let Some((_, b_value)) = b {
                    lock_guards.insert(key_a, b_value);
                }
            }
        }
    }
}
