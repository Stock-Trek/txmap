#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::creators::map_alice_bob_chuck,
            data::data::{ALICE, BOB, CHUCK},
        },
    };

    #[test]
    fn remove_multiple_keys() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .remove([ALICE.into(), BOB.into()])
            .get_copied(CHUCK.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(3)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_where_conditionally() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .remove_where([ALICE.into(), BOB.into(), CHUCK.into()], |_k, v| *v >= 2)
            .get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![Some(1), None, None]));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_only_keeps_specified() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .retain_only([ALICE.into(), BOB.into()])
            .get_copied(CHUCK.into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn retain_where_keeps_matching() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .retain_where([ALICE.into(), BOB.into(), CHUCK.into()], |_k, v| *v >= 2)
            .get_all_copied([ALICE.into(), BOB.into(), CHUCK.into()])
            .into_transaction();
        assert_eq!(
            tx.execute(),
            TxResult::Completed(vec![None, Some(2), Some(3)])
        );
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn param_remove() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .with_param::<()>()
            .remove([ALICE.into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn param_retain_only() {
        let map = map_alice_bob_chuck(1, 2, 3);
        let tx = map
            .transaction()
            .with_param::<()>()
            .retain_only([ALICE.into()])
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(Some(1)));
        assert_eq!(map.len(), 1);
    }
}
