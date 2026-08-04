pub(crate) const MISSING_LOCK_GUARD_ERROR: &str = "Missing lock guard";

/// The error type returned by [`TxMap::try_reserve`](crate::tx_map::TxMap::try_reserve).
///
/// Mirrors the shape of `std::collections::TryReserveError`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TryReserveError {
    /// The computed capacity exceeded the collection's maximum.
    CapacityOverflow,
    /// The memory allocator returned an error.
    AllocError {
        /// The layout of the allocation request that failed.
        layout: std::alloc::Layout,
    },
}

impl std::fmt::Display for TryReserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("memory allocation failed because ")?;
        let reason = match self {
            TryReserveError::CapacityOverflow => {
                "the computed capacity exceeded the collection's maximum"
            }
            TryReserveError::AllocError { .. } => "the memory allocator returned an error",
        };
        f.write_str(reason)
    }
}

impl std::error::Error for TryReserveError {}

/// The result of executing a transaction.
#[derive(Clone, PartialEq, Eq)]
pub enum TxResult<S> {
    /// The transaction completed successfully.
    Completed { state: S },
    /// A guard precondition was not met.
    RequirementNotMet {
        index: usize,
        requirement: String,
        state: S,
    },
}

impl<T> std::fmt::Debug for TxResult<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Completed { state } => write!(f, "Transaction completed. Result: {:?}", state),
            Self::RequirementNotMet {
                index,
                requirement,
                state,
            } => {
                write!(
                    f,
                    "Requirement at index [{}] not met: {}. State: {:?}",
                    index, requirement, state
                )
            }
        }
    }
}

impl<T> std::fmt::Display for TxResult<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Completed { state } => write!(f, "Transaction completed. Result: {}", state),
            Self::RequirementNotMet {
                index,
                requirement,
                state,
            } => {
                write!(
                    f,
                    "Requirement at index [{}] not met: {}. State: {}",
                    index, requirement, state
                )
            }
        }
    }
}
