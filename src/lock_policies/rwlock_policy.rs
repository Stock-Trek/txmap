use crate::lock_policies::lock_policy::LockPolicy;
use std::cell::UnsafeCell;

/// Lock policy using interior mutability for each shard.
///
/// Shard access is serialised by the map's atomic shard mask.
pub struct RwLockPolicy;

impl Default for RwLockPolicy {
    fn default() -> Self {
        Self
    }
}

impl LockPolicy for RwLockPolicy {
    type Lock<T> = UnsafeCell<T>;

    type ReadGuard<'guard, T>
        = &'guard T
    where
        Self: 'guard,
        T: 'guard;

    type WriteGuard<'guard, T>
        = &'guard mut T
    where
        Self: 'guard,
        T: 'guard;

    fn new<T>(value: T) -> Self::Lock<T> {
        UnsafeCell::new(value)
    }

    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T> {
        unsafe { &*lock.get() }
    }

    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T> {
        unsafe { &mut *lock.get() }
    }
}
