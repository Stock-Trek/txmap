use std::fmt;

pub type TxResult<T> = Result<T, TxError>;

pub enum TxError {
    UnmetPrerequisite(usize, String),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::UnmetPrerequisite(index, name) => {
                write!(f, "Unmet prerequisite at index [{}]: {}", index, name)
            }
        }
    }
}
