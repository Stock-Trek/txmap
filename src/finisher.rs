use crate::{
    finishers::finisher_trait::FinisherTrait, locks::lock_policy::LockPolicy, new_types::BitMask,
    shard::Shard,
};
use intmap::IntMap;
use std::marker::PhantomData;

pub(crate) struct Finisher<K, V, F>
where
    F: FinisherTrait<K, V>,
{
    finisher: F,
    _phantom_k: PhantomData<K>,
    _phantom_v: PhantomData<V>,
}

impl<K, V, F> Finisher<K, V, F>
where
    F: FinisherTrait<K, V>,
{
    pub fn new(finisher: F) -> Self {
        Self {
            finisher,
            _phantom_k: PhantomData,
            _phantom_v: PhantomData,
        }
    }
    pub fn guards_bitmask(&self) -> BitMask {
        self.finisher.guards_bitmask()
    }
    pub fn finish<L>(&self, mutex_guards: &IntMap<u8, L::WriteGuard<'_, Shard<K, V>>>) -> F::Output
    where
        L: LockPolicy,
    {
        self.finisher.to_result::<L>(mutex_guards)
    }
}
