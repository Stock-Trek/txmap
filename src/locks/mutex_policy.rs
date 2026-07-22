use crate::locks::lock_policy::LockPolicy;
use parking_lot::Mutex;

pub struct MutexPolicy;

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
