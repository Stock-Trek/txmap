use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxResult {
    Completed,
    UnmetPrerequisite(usize, String),
}

impl fmt::Display for TxResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed => write!(f, "Transaction was successful"),
            Self::UnmetPrerequisite(index, name) => {
                write!(f, "Unmet prerequisite at index [{}]: {}", index, name)
            }
        }
    }
}
