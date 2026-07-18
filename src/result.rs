use std::fmt;

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
