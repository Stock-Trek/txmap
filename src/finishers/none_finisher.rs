use crate::{
    finishers::finisher_trait::FinisherTrait, locks::lock_policy::LockPolicy, new_types::BitMask,
};
use hashbrown::HashTable;
use intmap::IntMap;

pub struct NoneFinisher;

impl<K, V> FinisherTrait<K, V> for NoneFinisher {
    type Output = ();

    fn guards_bitmask(&self) -> BitMask {
        BitMask::default()
    }
    fn to_result<'guards, L>(
        &self,
        _: &'guards IntMap<u8, L::WriteGuard<'_, HashTable<(K, V)>>>,
    ) -> Self::Output
    where
        L: LockPolicy,
    {
    }
}
