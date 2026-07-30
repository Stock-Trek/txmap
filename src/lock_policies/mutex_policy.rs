use crate::lock_policies::lock_policy::LockPolicy;
use parking_lot::Mutex;

/// Lock policy using [`parking_lot::Mutex`] for each shard.
///
/// This is the default policy. Both reads and writes acquire an
/// exclusive mutex lock, so there is no read concurrency within a
/// single shard. Use [`RwLockPolicy`](crate::lock_policies::rwlock_policy::RwLockPolicy)
/// when you expect many readers on the same shard.
pub struct MutexPolicy;

impl Default for MutexPolicy {
    fn default() -> Self {
        Self
    }
}

impl LockPolicy for MutexPolicy {
    type Lock<T> = Mutex<T>;

    type ReadGuard<'guard, T>
        = parking_lot::MutexGuard<'guard, T>
    where
        Self: 'guard,
        T: 'guard;

    type WriteGuard<'guard, T>
        = parking_lot::MutexGuard<'guard, T>
    where
        Self: 'guard,
        T: 'guard;

    fn new<T>(value: T) -> Self::Lock<T> {
        Mutex::new(value)
    }

    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T> {
        lock.lock()
    }

    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T> {
        lock.lock()
    }
}
