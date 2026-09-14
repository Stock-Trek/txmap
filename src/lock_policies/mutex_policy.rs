use crate::lock_policies::lock_policy::LockPolicy;
use std::cell::UnsafeCell;

/// Lock policy using interior mutability for each shard.
///
/// This is the default policy. Shard access is serialised by the map's
/// atomic shard mask.
pub struct MutexPolicy;

impl Default for MutexPolicy {
    fn default() -> Self {
        Self
    }
}

impl LockPolicy for MutexPolicy {
    type Lock<T> = UnsafeCell<T>;

    fn new<T>(value: T) -> Self::Lock<T> {
        UnsafeCell::new(value)
    }

    fn as_ptr<T>(lock: &Self::Lock<T>) -> *const T {
        lock.get()
    }

    fn as_mut_ptr<T>(lock: &Self::Lock<T>) -> *mut T {
        lock.get()
    }
}
