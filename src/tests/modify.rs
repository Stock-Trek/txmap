#[cfg(test)]
mod modify {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{
            creators::creators::{empty_map, map_alice, map_alice_bob},
            data::{ALICE, BOB},
        },
    };

    #[test]
    fn modify_existing_key() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 5)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(6)));
    }

    #[test]
    fn modify_missing_key_is_noop() {
        let map = empty_map();
        let tx = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v = 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
    }

    #[test]
    fn modify_peek_existing_key() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [_bob]| *v += 5)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(6)));
    }

    #[test]
    fn modify_peek_missing_key_is_noop() {
        let map = empty_map();
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [_bob]| *v = 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
    }

    #[test]
    fn modify_peek_can_use_peeked_values() {
        let map = map_alice_bob(1, 2);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [bob]| {
                *v += *bob.unwrap()
            })
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(3)));
    }
}
