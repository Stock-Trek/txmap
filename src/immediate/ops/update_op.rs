use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct UpdateOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub transform: Box<dyn Fn(&K, Option<&V>, &mut STATE) -> Option<V> + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for UpdateOp<'tx, K, V, STATE>
where
    K: Clone + Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (BitMask::ZERO, self.key.shard_index.bitmask())
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        if (self.key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_key, value)| value);
            let new_value = (self.transform)(&self.key.key, value_ref, state);
            match new_value {
                Some(v) => lock_guards.insert(&self.key, v),
                None => {
                    lock_guards.remove_entry(&self.key);
                }
            };
        } else {
            let value_ref = lock_guards
                .read
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_key, value)| value);
            let new_value = (self.transform)(&self.key.key, value_ref, state);
            match new_value {
                Some(v) => lock_guards.insert(&self.key, v),
                None => {
                    lock_guards.remove_entry(&self.key);
                }
            };
        }
    }
}
