#[cfg(test)]
mod tests {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{
            creators::creators::{empty_map, map_alice, map_alice_bob},
            data::data::{ALICE, BOB, CHUCK},
        },
    };

    #[test]
    fn modify_existing_key() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v += 5)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(6)));
    }

    #[test]
    fn modify_missing_key_is_noop() {
        let map = empty_map();
        let tx = map
            .transaction()
            .modify(ALICE.into(), |_k, v| *v = 42)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
    }

    #[test]
    fn modify_peek_existing_key() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [_bob]| *v += 5)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(6)));
    }

    #[test]
    fn modify_peek_missing_key_is_noop() {
        let map = empty_map();
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [_bob]| *v = 42)
            .get_copied(ALICE.into())
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
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(3)));
    }

    #[test]
    fn modify_peek_with_empty_peek_keys() {
        let map = empty_map();
        map.insert(ALICE.into(), 10);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [], |_k, v, []: [Option<&u64>; 0]| *v = 99)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(99)));
    }

    #[test]
    fn modify_peek_modifies_with_peek_values() {
        let map = empty_map();
        map.insert("target".into(), 100);
        map.insert("reference".into(), 50);
        let tx = map
            .transaction()
            .modify_peek("target".into(), ["reference".into()], |_k, v, [ref_val]| {
                if let Some(r) = ref_val {
                    *v += r;
                }
            })
            .get_copied("target".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(150)));
    }

    #[test]
    fn modify_peek_missing_target_is_noop() {
        let map = empty_map();
        map.insert("ref".into(), 99);
        let tx = map
            .transaction()
            .modify_peek("missing".into(), ["ref".into()], |_k, v, [_r]| *v = 0)
            .get_all_copied(["missing".into(), "ref".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, Some(99)]));
    }

    #[test]
    fn modify_peek_modifies_using_peeked_values() {
        let map = empty_map();
        map.insert(ALICE.into(), 100);
        map.insert(BOB.into(), 20);
        map.insert(CHUCK.into(), 3);
        let tx = map
            .transaction()
            .modify_peek(ALICE.into(), [BOB.into(), CHUCK.into()], |_k, v, [b, c]| {
                *v += *b.unwrap() + *c.unwrap();
            })
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(123)));
    }
}
