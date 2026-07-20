#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        tests::{
            creators::{empty_map, empty_typed_map, map_alice, map_alice_bob},
            data::{ALICE, BOB},
        },
    };

    #[test]
    fn param_transaction_basic() {
        let map = map_alice(0);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .modify(ALICE.into(), |_k, v, param| *v += param)
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&50), TxResult::Completed(Some(50)));
        assert_eq!(tx.execute(&30), TxResult::Completed(Some(80)));
    }

    #[test]
    fn param_requirement_not_met() {
        let map = empty_typed_map::<String, u64>();
        map.insert("funds".into(), 100);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .require("sufficient", ["funds".into()], |[v], min| {
                v.copied().unwrap_or(0) >= *min
            })
            .modify("funds".into(), |_k, v, _p| *v += 0)
            .into_transaction();
        assert_eq!(tx.execute(&50), TxResult::Completed(()));
        assert!(matches!(
            tx.execute(&200),
            TxResult::RequirementNotMet(0, _)
        ));
    }

    #[test]
    fn param_insert_with() {
        let map: TxMap<String, String> = empty_typed_map();
        let tx = map
            .transaction()
            .with_param::<String>()
            .insert_with(ALICE.into(), |_k, param| param.clone())
            .get(ALICE.into(), |_k, v| v.clone())
            .into_transaction();
        assert_eq!(
            tx.execute(&"hello".into()),
            TxResult::Completed(Some("hello".into()))
        );
    }

    #[test]
    fn param_map_op() {
        let map = map_alice(10);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update(ALICE.into(), |_k, v, mult| v.map(|x| x * mult))
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(30)));
    }

    #[test]
    fn param_remove_where() {
        let map = map_alice_bob(5, 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .remove_where([ALICE.into(), BOB.into()], |_k, v, threshold| {
                *v > *threshold
            })
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(Some(5)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_retain_where() {
        let map = map_alice_bob(5, 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain_where([ALICE.into(), BOB.into()], |_k, v, threshold| {
                *v >= *threshold
            })
            .get_copied(BOB.into())
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(Some(15)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_modify_peek() {
        let map = map_alice_bob(10, 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .modify_peek(ALICE.into(), [BOB.into()], |_k, v, [bob], mult| {
                *v = bob.copied().unwrap_or(0) * mult
            })
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(15)));
    }

    #[test]
    fn param_swap_value() {
        let map = map_alice_bob(1, 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .swap_value(ALICE.into(), BOB.into())
            .get_all_copied([ALICE.into(), BOB.into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(vec![Some(2), Some(1)]));
    }

    #[test]
    fn param_move_value() {
        let map = map_alice(42);
        let tx = map
            .transaction()
            .with_param::<()>()
            .move_value(ALICE.into(), BOB.into())
            .get_all_copied([ALICE.into(), BOB.into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(vec![None, Some(42)]));
    }

    #[test]
    fn param_get_all() {
        let map = map_alice(10);
        let tx = map
            .transaction()
            .with_param::<()>()
            .modify(ALICE.into(), |_k, v, _p| *v += 0)
            .modify(BOB.into(), |_k, v, _p| *v += 0)
            .get_all_copied([ALICE.into(), BOB.into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(vec![Some(10), None]));
    }

    #[test]
    fn param_insert_default() {
        let map = empty_map();
        let tx = map
            .transaction()
            .with_param::<()>()
            .insert_default(ALICE.into())
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(Some(0)));
    }

    #[test]
    fn param_update_peek() {
        let map = map_alice_bob(10, 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update_peek(
                ALICE.into(),
                |_k, v, [bob], mult| v.map(|x| (x + bob.unwrap_or(&0)) * mult),
                [BOB.into()],
            )
            .get_copied(ALICE.into())
            .into_transaction();
        assert_eq!(tx.execute(&2), TxResult::Completed(Some(30)));
    }
}
