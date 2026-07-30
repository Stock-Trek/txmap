use crate::lock_policies::lock_policy::LockPolicy;
use parking_lot::RwLock;

/// Lock policy using [`parking_lot::RwLock`] for each shard.
///
/// Allows concurrent readers on the same shard. Writes are exclusive.
/// Useful for read-heavy workloads.
pub struct RwLockPolicy;

impl Default for RwLockPolicy {
    fn default() -> Self {
        Self
    }
}

impl LockPolicy for RwLockPolicy {
    type Lock<T> = RwLock<T>;

    type ReadGuard<'guard, T>
        = parking_lot::RwLockReadGuard<'guard, T>
    where
        Self: 'guard,
        T: 'guard;

    type WriteGuard<'guard, T>
        = parking_lot::RwLockWriteGuard<'guard, T>
    where
        Self: 'guard,
        T: 'guard;

    fn new<T>(value: T) -> Self::Lock<T> {
        RwLock::new(value)
    }

    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T> {
        lock.read()
    }

    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T> {
        lock.write()
    }
}
