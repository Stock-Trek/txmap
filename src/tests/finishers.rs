#[cfg(test)]
mod tests {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{
            creators::empty_map,
            data::{ALICE, BOB, CHUCK},
        },
    };

    #[test]
    fn get_returns_transformed_value() {
        let map = empty_map();
        map.insert(ALICE.into(), 7);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 3)
            .get(ALICE.into(), |_k, v| *v * 2)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_returns_option_via_map_finisher() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v *= 2)
            .get_copied(ALICE.into())
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_all_returns_multiple_values() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        map.insert(BOB.into(), 20);
        let result = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 0)
            .modify(BOB.into(), |_k, v| *v += 0)
            .modify(CHUCK.into(), |_k, v| *v += 0)
            .get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()])
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20), None]));
    }
}
