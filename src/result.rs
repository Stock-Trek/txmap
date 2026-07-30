pub(crate) const MISSING_LOCK_GUARD_ERROR: &str = "Missing lock guard";

/// The result of executing a transaction.
#[derive(Clone, PartialEq, Eq)]
pub enum TxResult<T> {
    /// The transaction completed successfully.
    Completed(T),
    /// A guard precondition was not met.
    RequirementNotMet(usize, String, T),
}

impl<T> std::fmt::Debug for TxResult<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Completed(state) => write!(f, "Transaction completed. Result: {:?}", state),
            Self::RequirementNotMet(index, name, state) => {
                write!(
                    f,
                    "Requirement at index [{}] not met: {}. State: {:?}",
                    index, name, state
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
            Self::Completed(state) => write!(f, "Transaction completed. Result: {}", state),
            Self::RequirementNotMet(index, name, state) => {
                write!(
                    f,
                    "Requirement at index [{}] not met: {}. State: {}",
                    index, name, state
                )
            }
        }
    }
}
