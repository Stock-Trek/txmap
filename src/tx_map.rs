use crate::{
    builders::stem_builder::TxStemBuilder, custodian::Custodian, indexer::Indexer,
    result::MISSING_MUTEX_GUARD_ERROR, shard_count::ShardCount,
};
use std::hash::{DefaultHasher, Hash};

pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: ShardCount) -> Self {
        let indexer = Indexer {
            shard_count: u8::from(shard_count),
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        Self {
            indexer,
            custodian: Custodian::new(shard_count),
        }
    }
    pub fn clear(&self) {
        let all_guards = self.custodian.all_guards();
        for mut mutex_guard in all_guards {
            mutex_guard.1.clear();
        }
    }
    pub fn len(&self) -> usize {
        let mut total_length = 0;
        let all_guards = self.custodian.all_guards();
        for mutex_guard in all_guards {
            total_length += mutex_guard.1.len();
        }
        total_length
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let shard_index = self.indexer.index(&key);
        let mut mutex_guards = self.custodian.guards(1 << shard_index);
        let mutex_guard = mutex_guards
            .get_mut(shard_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard.insert(key, value)
    }
    pub fn remove(&self, key: &K) -> Option<V> {
        let shard_index = self.indexer.index(key);
        let mut mutex_guards = self.custodian.guards(1 << shard_index);
        let mutex_guard = mutex_guards
            .get_mut(shard_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard.remove(key)
    }
    pub fn get_with<T, R>(&self, key: &K, transform: T) -> Option<R>
    where
        T: FnOnce(&V) -> R,
    {
        let shard_index = self.indexer.index(key);
        let mut mutex_guards = self.custodian.guards(1 << shard_index);
        let mutex_guard = mutex_guards
            .get_mut(shard_index)
            .expect(MISSING_MUTEX_GUARD_ERROR);
        mutex_guard.get(key).map(transform)
    }
    pub fn fold<T, R, C, A>(&self, initial: R, convert: C, accumulate: A) -> R
    where
        C: Fn(&K, &V) -> Option<T>,
        A: Fn(R, T) -> R,
    {
        self.custodian
            .all_guards()
            .iter()
            .flat_map(|guard| guard.1.iter())
            .filter_map(|(key, value)| convert(key, value))
            .fold(initial, accumulate)
    }
    pub fn transaction<'txmap>(&'txmap self) -> TxStemBuilder<'txmap, K, V> {
        TxStemBuilder {
            indexer: self.indexer,
            custodian: &self.custodian,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{indexer::Indexer, prelude::*};
    use std::hash::DefaultHasher;

    #[test]
    pub fn transfer() {
        let db: TxMap<User, Funds> = TxMap::new(ShardCount::_8);
        let bob = User {
            first_name: "Bob".into(),
            last_name: "Bobson".into(),
        };
        let tim = User {
            first_name: "Tim".into(),
            last_name: "Timson".into(),
        };
        let pam = User {
            first_name: "Pam".into(),
            last_name: "Pamson".into(),
        };
        db.transaction()
            .update(tim.clone(), |_t, _f| {
                Some(Funds {
                    usd_and_cents: 150,
                    sterling_and_pence: 0,
                })
            })
            .into_transaction()
            .execute();
        let send_1_usd_from_bob_to_tim = db
            .transaction()
            .require("Has available funds", [tim.clone()], |[tim_funds]| {
                tim_funds.is_some_and(|f| f.usd_and_cents > 100)
            })
            .update(tim.clone(), |_t, tim_funds| {
                Some(Funds {
                    sterling_and_pence: tim_funds.unwrap().sterling_and_pence,
                    usd_and_cents: tim_funds.unwrap().usd_and_cents - 100,
                })
            })
            .update(bob.clone(), |_b, bob_funds| {
                Some(bob_funds.map_or(
                    Funds {
                        usd_and_cents: 100,
                        sterling_and_pence: 0,
                    },
                    |f| Funds {
                        usd_and_cents: f.usd_and_cents + 100,
                        sterling_and_pence: f.sterling_and_pence,
                    },
                ))
            })
            .into_transaction();
        assert_eq!(
            send_1_usd_from_bob_to_tim.execute(),
            TxResult::Completed(())
        );
        assert_ne!(
            send_1_usd_from_bob_to_tim.execute(),
            TxResult::Completed(())
        );

        let send_x_usd_from_bob_to_tim = db
            .transaction()
            .with_param::<Transfer>()
            .require(
                "Has available funds",
                [tim.clone()],
                |[tim_funds], params| {
                    tim_funds.is_some_and(|f| f.usd_and_cents >= params.usd_and_cents)
                },
            )
            .insert_default_if_absent(bob.clone())
            .modify(bob.clone(), |_bob, funds, params| {
                funds.usd_and_cents -= params.usd_and_cents
            })
            .modify(tim.clone(), |_tim, funds, params| {
                funds.usd_and_cents += params.usd_and_cents
            })
            .get_all([bob.clone(), tim.clone()], |_user, funds| {
                funds.usd_and_cents
            })
            .into_transaction();
        assert_eq!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 40 }),
            TxResult::Completed(vec![Some(60), Some(90)])
        );
        assert_ne!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 20 }),
            TxResult::Completed(vec![Some(40), Some(60)])
        );

        let add_100_usd_to_bob_if_exists = db
            .transaction()
            .modify(bob.clone(), |_b, bob_funds| {
                bob_funds.usd_and_cents += 100;
            })
            .into_transaction();
        assert_eq!(
            add_100_usd_to_bob_if_exists.execute(),
            TxResult::Completed(())
        );
        assert_eq!(
            add_100_usd_to_bob_if_exists.execute(),
            TxResult::Completed(())
        );

        let add_123_to_pam = db
            .transaction()
            .insert_default_if_absent(pam.clone())
            .modify(pam.clone(), |_p, pam_funds| {
                pam_funds.usd_and_cents += 123;
            })
            .get(pam.clone(), |_user, funds| funds.usd_and_cents)
            .into_transaction();
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(Some(123)));
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(Some(246)));
    }

    #[test]
    fn indexer_distributes_across_shards() {
        let indexer = Indexer {
            shard_count: 8,
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        let mut seen = std::collections::HashSet::new();
        for i in 0..1000u64 {
            seen.insert(indexer.index(&i));
        }
        // With 1000 keys and 8 shards, we should hit most shards
        assert!(
            seen.len() >= 4,
            "should hit at least 4 shards, got {}",
            seen.len()
        );
    }

    // ── Transaction: get / get_all finishers ───────────────────────────────────

    #[test]
    fn get_returns_transformed_value() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 7);
        let result = map
            .transaction()
            .modify("k".into(), |_k, v| *v += 3)
            .get("k".into(), |_k, v| *v * 2)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_returns_option_via_map_finisher() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        let result = map
            .transaction()
            .modify("k".into(), |_k, v| *v *= 2)
            .get("k".into(), |_k, v| *v)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(Some(20)));
    }

    #[test]
    fn get_all_returns_multiple_values() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        map.insert("b".into(), 20);
        let result = map
            .transaction()
            .modify("a".into(), |_k, v| *v += 0)
            .modify("b".into(), |_k, v| *v += 0)
            .modify("c".into(), |_k, v| *v += 0)
            .get_all(["a".into(), "b".into(), "c".into()], |_k, v| *v)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20), None]));
    }

    // ── Transaction: batch ops ─────────────────────────────────────────────────

    #[test]
    fn remove_multiple_keys() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove(["a".into(), "c".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(2));
    }

    #[test]
    fn remove_where_conditionally() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove_where(["a".into(), "b".into(), "c".into()], |_k, v| *v >= 2)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(1));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_only_keeps_specified() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .retain_only(["a".into(), "b".into()])
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_with(&"c".into(), |v| *v), None);
    }

    #[test]
    fn retain_where_keeps_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        map.insert("b".into(), 20);
        map.insert("c".into(), 30);
        let tx = map
            .transaction()
            .retain_where(["a".into(), "b".into(), "c".into()], |_k, v| *v >= 20)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get_with(&"a".into(), |v| *v), None);
    }

    // ── Transaction: global ops ────────────────────────────────────────────────

    #[test]
    fn clear_via_transaction() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map.transaction().clear().into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert!(map.is_empty());
    }

    #[test]
    fn remove_if_removes_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .remove_if(|_k, v| *v > 1)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn retain_keeps_matching() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .retain(|_k, v| *v % 2 == 0)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(2));
    }

    // ── Transaction: chained ops ───────────────────────────────────────────────

    #[test]
    fn chained_modify_and_get() {
        let map: TxMap<String, Counter> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .insert_default("ctr".into())
            .modify("ctr".into(), |_k, c| c.value += 1)
            .modify("ctr".into(), |_k, c| c.value += 1)
            .get("ctr".into(), |_k, c| c.value)
            .into_transaction();
        let result = tx.execute();
        assert_eq!(result, TxResult::Completed(Some(2)));
    }

    #[test]
    fn chained_ops_on_multiple_keys() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .insert_default("x".into())
            .insert_default("y".into())
            .modify("x".into(), |_k, v| *v += 10)
            .modify("y".into(), |_k, v| *v += 20)
            .get_all(["x".into(), "y".into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(vec![Some(10), Some(20)]));
    }

    // ── Parameterized transactions ─────────────────────────────────────────────

    #[test]
    fn param_transaction_basic() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .insert_default("k".into())
            .modify("k".into(), |_k, v, param| *v += param)
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&50), TxResult::Completed(Some(50)));
        assert_eq!(tx.execute(&30), TxResult::Completed(Some(80)));
    }

    #[test]
    fn param_requirement_not_met() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
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
        let map: TxMap<String, String> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .with_param::<String>()
            .insert_with("k".into(), |_k, param| param.clone())
            .into_transaction();
        assert_eq!(tx.execute(&"hello".into()), TxResult::Completed(()));
        assert_eq!(
            map.get_with(&"k".into(), |v| v.clone()),
            Some("hello".into())
        );
    }

    #[test]
    fn param_map_op() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update("k".into(), |_k, v, mult| v.map(|x| x * mult))
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(30)));
    }

    #[test]
    fn param_remove_where() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 5);
        map.insert("b".into(), 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .remove_where(["a".into(), "b".into()], |_k, v, threshold| *v > *threshold)
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(5));
    }

    #[test]
    fn param_retain_where() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 5);
        map.insert("b".into(), 15);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain_where(["a".into(), "b".into()], |_k, v, threshold| {
                *v >= *threshold
            })
            .into_transaction();
        assert_eq!(tx.execute(&10), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(15));
    }

    #[test]
    fn param_remove_if_global() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .remove_if(|_k, v, max| *v > *max)
            .into_transaction();
        assert_eq!(tx.execute(&1), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_retain_global() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        map.insert("b".into(), 20);
        map.insert("c".into(), 30);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .retain(|_k, v, min| *v >= *min)
            .into_transaction();
        assert_eq!(tx.execute(&25), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    // ── peek variants ─────────────────────────────────────────────────────────

    #[test]
    fn modify_peek_modifies_with_peek_values() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("target".into(), 100);
        map.insert("reference".into(), 50);
        let tx = map
            .transaction()
            .modify_peek("target".into(), ["reference".into()], |_k, v, [ref_val]| {
                if let Some(r) = ref_val {
                    *v += r;
                }
            })
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"target".into(), |v| *v), Some(150));
    }

    #[test]
    fn modify_peek_missing_target_is_noop() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("ref".into(), 99);
        let tx = map
            .transaction()
            .modify_peek("missing".into(), ["ref".into()], |_k, v, [_r]| *v = 0)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"ref".into(), |v| *v), Some(99));
    }

    #[test]
    fn modify_peek_modifies_using_peeked_values() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 100);
        map.insert("b".into(), 20);
        map.insert("c".into(), 3);
        let tx = map
            .transaction()
            .modify_peek("a".into(), ["b".into(), "c".into()], |_k, v, [b, c]| {
                *v += *b.unwrap() + *c.unwrap();
            })
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(123));
    }

    #[test]
    fn update_peek_modifies_based_on_peek() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .update_peek(
                "k".into(),
                |_k, v, [p]| v.map(|x| x + p.unwrap_or(&0)),
                ["p".into()],
            )
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(15));
    }

    // ── Edge cases and adversarial tests ───────────────────────────────────────

    #[test]
    fn large_number_of_keys() {
        let map: TxMap<u64, u64> = TxMap::new(ShardCount::_128);
        for i in 0..10_000 {
            map.insert(i, i * 3);
        }
        assert_eq!(map.len(), 10_000);
        // Verify a few values
        for i in (0..10_000).step_by(1000) {
            assert_eq!(map.get_with(&i, |v| *v), Some(i * 3));
        }
    }

    #[test]
    fn duplicate_keys_dont_cause_issues() {
        let map: TxMap<String, String> = TxMap::new(ShardCount::_8);
        let key: String = "same".into();
        for i in 0..100 {
            map.insert(key.clone(), format!("v{i}"));
        }
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&key, |v| v.clone()), Some("v99".into()));
    }

    #[test]
    fn empty_key_works() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("".into(), 1);
        assert_eq!(map.get_with(&"".into(), |v| *v), Some(1));
        let tx = map
            .transaction()
            .modify("".into(), |_k, v| *v += 1)
            .get("".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(Some(2)));
    }

    #[test]
    fn transaction_on_empty_map() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let result = map
            .transaction()
            .modify("k".into(), |_k, v| *v = 42)
            .get("k".into(), |_k, v| *v)
            .into_transaction()
            .execute();
        assert_eq!(result, TxResult::Completed(None));
    }

    #[test]
    fn mixed_ops_in_one_transaction() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .insert_default("a".into())
            .insert_default("b".into())
            .insert_default("c".into())
            .modify("a".into(), |_k, v| *v = 10)
            .modify("b".into(), |_k, v| *v = 20)
            .update("c".into(), |_k, _v| Some(30))
            .get_all(["a".into(), "b".into(), "c".into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(
            tx.execute(),
            TxResult::Completed(vec![Some(10), Some(20), Some(30)])
        );
    }

    #[test]
    fn chain_many_ops() {
        let map: TxMap<u64, u64> = TxMap::new(ShardCount::_8);
        // Build transaction with multiple ops chained manually
        let tx = map
            .transaction()
            .insert_default(0)
            .insert_default(1)
            .insert_default(2)
            .insert_default(3)
            .insert_default(4)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn chain_many_ops_with_params() {
        let map: TxMap<u64, u64> = TxMap::new(ShardCount::_8);
        // Use a single transaction that modifies via with_param
        let tx = map
            .transaction()
            .with_param::<Vec<u64>>()
            .insert_default(0)
            .insert_default(1)
            .modify(0, |_k, v, p| *v = p[0])
            .modify(1, |_k, v, p| *v = p[1])
            .get_all([0, 1], |_k, v| *v)
            .into_transaction();
        let result = tx.execute(&vec![10, 20]);
        assert_eq!(result, TxResult::Completed(vec![Some(10), Some(20)]));
    }

    #[test]
    fn clear_then_reinsert() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.clear();
        assert!(map.is_empty());
        map.insert("a".into(), 2);
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(2));
    }

    #[test]
    fn remove_if_empty_map() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .remove_if(|_k, _v| true)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn retain_all_on_empty_map() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map.transaction().retain(|_k, _v| false).into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
    }

    #[test]
    fn huge_string_keys() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let big_key = "x".repeat(10_000);
        map.insert(big_key.clone(), 42);
        assert_eq!(map.get_with(&big_key, |v| *v), Some(42));
    }

    #[test]
    fn swap_value_same_key() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 7);
        let tx = map
            .transaction()
            .swap_value("k".into(), "k".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(7));
    }

    #[test]
    fn move_value_to_self() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 7);
        let tx = map
            .transaction()
            .move_value("k".into(), "k".into())
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(7));
    }

    #[test]
    fn modify_peek_with_empty_peek_keys() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        let tx = map
            .transaction()
            .modify_peek("k".into(), [], |_k, v, []: [Option<&u64>; 0]| *v = 99)
            .into_transaction();
        assert_eq!(tx.execute(), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(99));
    }

    #[test]
    fn param_modify_peek() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .modify_peek("k".into(), ["p".into()], |_k, v, [p], mult| {
                *v = p.copied().unwrap_or(0) * mult
            })
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&3), TxResult::Completed(Some(15)));
    }

    #[test]
    fn param_swap_value() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .swap_value("a".into(), "b".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(2));
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(1));
    }

    #[test]
    fn param_move_value() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 42);
        let tx = map
            .transaction()
            .with_param::<()>()
            .move_value("a".into(), "b".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"b".into(), |v| *v), Some(42));
        assert_eq!(map.get_with(&"a".into(), |v| *v), None);
    }

    #[test]
    fn param_clear() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        let tx = map
            .transaction()
            .with_param::<()>()
            .clear()
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert!(map.is_empty());
    }

    #[test]
    fn param_remove() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .remove(["a".into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn param_retain_only() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 1);
        map.insert("b".into(), 2);
        let tx = map
            .transaction()
            .with_param::<()>()
            .retain_only(["a".into()])
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get_with(&"a".into(), |v| *v), Some(1));
    }

    #[test]
    fn param_get_all() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("a".into(), 10);
        let tx = map
            .transaction()
            .with_param::<()>()
            .modify("a".into(), |_k, v, _p| *v += 0)
            .modify("b".into(), |_k, v, _p| *v += 0)
            .get_all(["a".into(), "b".into()], |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(vec![Some(10), None]));
    }

    #[test]
    fn param_insert_default() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        let tx = map
            .transaction()
            .with_param::<()>()
            .insert_default("k".into())
            .into_transaction();
        assert_eq!(tx.execute(&()), TxResult::Completed(()));
        assert_eq!(map.get_with(&"k".into(), |v| *v), Some(0));
    }

    #[test]
    fn param_update_peek() {
        let map: TxMap<String, u64> = TxMap::new(ShardCount::_8);
        map.insert("k".into(), 10);
        map.insert("p".into(), 5);
        let tx = map
            .transaction()
            .with_param::<u64>()
            .update_peek(
                "k".into(),
                |_k, v, [p], mult| v.map(|x| (x + p.unwrap_or(&0)) * mult),
                ["p".into()],
            )
            .get("k".into(), |_k, v| *v)
            .into_transaction();
        assert_eq!(tx.execute(&2), TxResult::Completed(Some(30)));
    }
}
