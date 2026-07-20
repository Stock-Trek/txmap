pub(crate) const INCORRECT_GUARD_VALUES_LENGTH: &str = "Incorrect guard values length";
pub(crate) const INCORRECT_PEEK_VALUES_LENGTH: &str = "Incorrect peek values length";
pub(crate) const MISSING_MUTEX_GUARD_ERROR: &str = "Missing mutex guard";

#[derive(Clone, PartialEq, Eq)]
pub enum TxResult<T> {
    Completed(T),
    RequirementNotMet(usize, String),
}

impl<T> std::fmt::Debug for TxResult<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Completed(result) => write!(f, "Transaction completed. Result: {:?}", result),
            Self::RequirementNotMet(index, name) => {
                write!(f, "Requirement at index [{}] not met: {}", index, name)
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
            Self::Completed(result) => write!(f, "Transaction completed. Result: {}", result),
            Self::RequirementNotMet(index, name) => {
                write!(f, "Requirement at index [{}] not met: {}", index, name)
            }
        }
    }
}
