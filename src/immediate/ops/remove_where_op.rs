use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct RemoveWhereOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub condition: Box<dyn Fn(&K, &V, &mut STATE) -> bool + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for RemoveWhereOp<'tx, K, V, STATE>
where
    K: Hash + Eq + 'tx,
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
            if let Some(v) = value_ref
                && (self.condition)(&self.key.key, v, state)
            {
                lock_guards.remove_entry(&self.key);
            }
        } else {
            let value_ref = lock_guards
                .read
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_key, value)| value);
            if let Some(v) = value_ref
                && (self.condition)(&self.key.key, v, state)
            {
                lock_guards.remove_entry(&self.key);
            }
        }
    }
}
