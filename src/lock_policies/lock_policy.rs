/// Locking policy for shard-level synchronisation.
///
/// Implementations determine the storage used for each shard. Two built-in
/// policies are provided:
/// [`MutexPolicy`](crate::lock_policies::mutex_policy::MutexPolicy) and
/// [`RwLockPolicy`](crate::lock_policies::rwlock_policy::RwLockPolicy).
///
/// Shard exclusion itself is enforced by the map's atomic shard mask, so the
/// policy only needs to expose the raw pointer to a shard's value.
pub trait LockPolicy {
    /// The lock type wrapping a shard.
    type Lock<T>;

    /// Create a new locked shard.
    fn new<T>(value: T) -> Self::Lock<T>;
    /// Returns a shared pointer to the locked value.
    fn as_ptr<T>(lock: &Self::Lock<T>) -> *const T;
    /// Returns a mutable pointer to the locked value.
    fn as_mut_ptr<T>(lock: &Self::Lock<T>) -> *mut T;
}
