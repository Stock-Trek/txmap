#[cfg(test)]
mod swap_value {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::{map_alice, map_alice_bob},
            data::data::{ALICE, BOB},
        },
    };

    #[test]
    fn swap_value_exchanges_values() {
        let map = map_alice_bob(1, 2);
        let tx = map
            .transaction()
            .swap_value(ALICE.into(), BOB.into())
            .get_all([ALICE.into(), BOB.into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![Some(2), Some(1)]));
    }

    #[test]
    fn swap_with_missing_value() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .swap_value(ALICE.into(), BOB.into())
            .get_all([ALICE.into(), BOB.into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, Some(1)]));
    }
}
