use crate::{locks::lock_policy::LockPolicy, new_types::BitMask};
use hashbrown::HashTable;
use intmap::IntMap;

pub(crate) trait OpTrait<L, K, V, P> {
    fn guards_bitmask(&self) -> BitMask;
    fn apply<'guards>(
        &self,
        mutex_guards: &'guards mut IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
        params: &P,
    ) where
        L: LockPolicy;
}
