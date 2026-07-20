#[cfg(test)]
mod insert {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{
            creators::creators::{empty_map, map_alice},
            data::ALICE,
        },
    };

    #[test]
    fn insert_with_creates_entry() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_with(ALICE.into(), |_key| 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(42)));
    }

    #[test]
    fn insert_with_overwrites_existing() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .insert_with(ALICE.into(), |_key| 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(42)));
    }

    #[test]
    fn insert_with_if_absent_creates_entry() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_with_if_absent(ALICE.into(), |_key| 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(42)));
    }

    #[test]
    fn insert_with_if_absent_does_not_overwrite_existing() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .insert_with_if_absent(ALICE.into(), |_key| 42)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(1)));
    }

    #[test]
    fn insert_default_creates_default_entry() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_default(ALICE.into())
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(0)));
    }

    #[test]
    fn insert_default_overwrites_existing() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .insert_default(ALICE.into())
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(0)));
    }

    #[test]
    fn insert_default_if_absent_creates_default_entry() {
        let map = empty_map();
        let tx = map
            .transaction()
            .insert_default_if_absent(ALICE.into())
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(0)));
    }

    #[test]
    fn insert_default_if_absent_does_not_overwrite_existing() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .insert_default_if_absent(ALICE.into())
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(1)));
    }
}
