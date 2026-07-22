use crate::{locks::lock_policy::LockPolicy, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;

pub trait FinisherTrait<K, V> {
    type Output;

    fn guards_bitmask(&self) -> BitMask;
    fn to_result<'guards, L>(
        &self,
        mutex_guards: &'guards IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Self::Output
    where
        L: LockPolicy;
}
