use std::ops::{Deref, DerefMut};

pub trait LockPolicy {
    type Lock<T>;

    type ReadGuard<'guard, T>: Deref<Target = T>
    where
        Self: 'guard,
        T: 'guard;

    type WriteGuard<'guard, T>: DerefMut<Target = T>
    where
        Self: 'guard,
        T: 'guard;

    fn new<T>(value: T) -> Self::Lock<T>;
    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T>;
    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T>;
}
