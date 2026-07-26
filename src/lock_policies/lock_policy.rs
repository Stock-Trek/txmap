use std::ops::{Deref, DerefMut};

pub trait LockPolicy {
    type Lock<T>;

    type ReadGuard<'lock, T>: Deref<Target = T>
    where
        Self: 'lock,
        T: 'lock;

    type WriteGuard<'lock, T>: DerefMut<Target = T>
    where
        Self: 'lock,
        T: 'lock;

    fn new<T>(value: T) -> Self::Lock<T>;
    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T>;
    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T>;
}
