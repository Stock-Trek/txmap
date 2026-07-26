use crate::{
    immediate::ops::op_trait::OpTrait, key::TxKey, lock_guards::LockGuards,
    lock_policies::lock_policy::LockPolicy, new_types::BitMask, result::MISSING_LOCK_GUARD_ERROR,
};
use std::hash::Hash;

pub(crate) struct GetOp<'tx, K, V, STATE>
where
    K: Hash + Eq,
{
    pub key: TxKey<K>,
    #[allow(clippy::type_complexity)]
    pub get: Box<dyn Fn(&K, Option<&V>, &mut STATE) + 'tx>,
}

impl<'tx, K, V, L, STATE> OpTrait<K, V, L, STATE> for GetOp<'tx, K, V, STATE>
where
    K: Hash + Eq + 'tx,
    V: 'tx,
    L: LockPolicy + 'tx,
{
    fn read_write_bitmasks(&self) -> (BitMask, BitMask) {
        (self.key.shard_index.bitmask(), BitMask::ZERO)
    }
    fn apply(&self, lock_guards: &mut LockGuards<'_, K, V, L>, state: &mut STATE) {
        if (self.key.shard_index.bitmask() & lock_guards.write_bitmask) != BitMask::ZERO {
            let value_ref = lock_guards
                .write
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_, value)| value);
            (self.get)(&self.key.key, value_ref, state)
        } else {
            let value_ref = lock_guards
                .read
                .get(self.key.shard_index.0)
                .expect(MISSING_LOCK_GUARD_ERROR)
                .find(self.key.hash_code.0, |entry| entry.0 == self.key.key)
                .map(|(_, value)| value);
            (self.get)(&self.key.key, value_ref, state)
        }
    }
}
