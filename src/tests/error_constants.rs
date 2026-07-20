#[cfg(test)]
mod error_constants {
    use crate::result::{
        INCORRECT_GUARD_VALUES_LENGTH, INCORRECT_PEEK_VALUES_LENGTH, MISSING_MUTEX_GUARD_ERROR,
        TxResult,
    };

    #[test]
    fn error_constants_are_defined() {
        assert!(!MISSING_MUTEX_GUARD_ERROR.is_empty());
        assert!(!INCORRECT_GUARD_VALUES_LENGTH.is_empty());
        assert!(!INCORRECT_PEEK_VALUES_LENGTH.is_empty());
    }

    #[test]
    fn display_tx_result() {
        let completed: TxResult<i32> = TxResult::Completed(42);
        let display = format!("{completed}");
        assert!(display.contains("42"), "display string should contain 42");
    }
}
