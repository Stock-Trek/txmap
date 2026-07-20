#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::{map_alice, map_alice_bob},
            data::data::{ALICE, BOB},
        },
    };

    #[test]
    fn move_existing_value() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .move_value(ALICE.into(), BOB.into())
            .get_all([ALICE.into(), BOB.into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, Some(1)]));
    }

    #[test]
    fn move_value_overwrites_existing() {
        let map = map_alice_bob(1, 2);
        let tx = map
            .transaction()
            .move_value(ALICE.into(), BOB.into())
            .get_all([ALICE.into(), BOB.into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, Some(1)]));
    }

    #[test]
    fn move_none_removes_existing() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .move_value(BOB.into(), ALICE.into())
            .get_all_copied([BOB.into(), ALICE.into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![None, None]));
    }
}
