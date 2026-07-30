use std::ops::{Deref, DerefMut};

/// Locking policy for shard-level synchronisation.
///
/// Implementations determine the type of lock used per shard and how
/// read/write guards are acquired. Two built-in policies are provided:
/// [`MutexPolicy`](crate::lock_policies::mutex_policy::MutexPolicy) and
/// [`RwLockPolicy`](crate::lock_policies::rwlock_policy::RwLockPolicy).
pub trait LockPolicy {
    /// The lock type wrapping a shard.
    type Lock<T>;

    /// Read guard type (must deref to `T`).
    type ReadGuard<'lock, T>: Deref<Target = T>
    where
        Self: 'lock,
        T: 'lock;

    /// Write guard type (must deref-mut to `T`).
    type WriteGuard<'lock, T>: DerefMut<Target = T>
    where
        Self: 'lock,
        T: 'lock;

    /// Create a new locked shard.
    fn new<T>(value: T) -> Self::Lock<T>;
    /// Acquire a read guard.
    fn read<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::ReadGuard<'lock, T>;
    /// Acquire a write guard.
    fn write<'lock, T>(lock: &'lock Self::Lock<T>) -> Self::WriteGuard<'lock, T>;
}
