#[cfg(test)]
mod tests {
    use crate::{
        builders::builder_traits::{IntoTransaction, TxOpBuilder, TxResultBuilder},
        result::TxResult,
        tests::{creators::creators::map_alice, data::data::ALICE},
    };

    #[test]
    fn update_overwrites_when_returning_some() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, _v| Some(42))
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(42)));
    }

    #[test]
    fn update_removes_when_returning_none() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, _v| None)
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(None));
    }

    #[test]
    fn update_transforms_existing_value() {
        let map = map_alice(1);
        let tx = map
            .transaction()
            .update(ALICE.into(), |_k, v| v.map(|x| x * 2))
            .get(ALICE.into(), |_, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(2)));
    }
}
