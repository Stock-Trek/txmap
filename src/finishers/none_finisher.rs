use crate::finishers::finisher_trait::FinisherTrait;
use hashbrown::HashMap;
use intmap::IntMap;
use parking_lot::MutexGuard;
use std::hash::Hash;

pub struct NoneFinisher;

impl<K, V> FinisherTrait<K, V> for NoneFinisher
where
    K: Clone + Hash + Eq,
{
    type Output = ();

    fn to_result(&self, _: &IntMap<u8, MutexGuard<'_, HashMap<K, V>>>) -> Self::Output {}
}
