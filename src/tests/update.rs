#[cfg(test)]
mod tests {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{
            creators::{map_alice, map_alice_bob},
            data::{ALICE, BOB},
        },
    };

    #[test]
    fn update_overwrites_when_returning_some() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, _v| Some(42))
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(42)));
    }

    #[test]
    fn update_removes_when_returning_none() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, _v| None)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
    }

    #[test]
    fn update_transforms_existing_value() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, v| v.map(|x| x * 2))
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(2)));
    }

    #[test]
    fn update_peek_modifies_based_on_peek() {
        let map = map_alice_bob(10, 5);
        let tx = map
            .transaction()
            .update_peek(ALICE.into(), [BOB.into()], |_k, v, [p]| {
                v.map(|x| x + p.unwrap_or(&0))
            })
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(15)));
    }
}
