use std::fmt;

pub(crate) const INCORRECT_GUARD_VALUES_LENGTH: &str = "Incorrect guard values length";
pub(crate) const INCORRECT_PEEK_VALUES_LENGTH: &str = "Incorrect peek values length";
pub(crate) const MISSING_MUTEX_GUARD_ERROR: &str = "Missing mutex guard";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxResult {
    Completed,
    ConditionNotMet(usize, String),
}

impl fmt::Display for TxResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed => write!(f, "Transaction completed"),
            Self::ConditionNotMet(index, name) => {
                write!(f, "Condition not met at index [{}]: {}", index, name)
            }
        }
    }
}
