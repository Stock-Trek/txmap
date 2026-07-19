use crate::{
    custodian::Custodian, indexer::Indexer, result::MISSING_MUTEX_GUARD_ERROR,
    shard_count::ShardCount, tx_stem_builder::TxStemBuilder,
};
use std::hash::{DefaultHasher, Hash};

pub struct TxMap<K, V>
where
    K: Clone + Hash + Eq,
{
    indexer: Indexer,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Clone + Hash + Eq,
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
        mutex_guard.get(key).map(|v| transform(v))
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
    use crate::{
        parameterized_builder_traits::{
            IntoParameterizedTransaction, WithParameterizedOperation, WithParameterizedPrerequisite,
        },
        prelude::*,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct User {
        first_name: String,
        last_name: String,
    }
    struct Funds {
        usd_and_cents: u64,
        sterling_and_pence: u64,
    }
    struct Transfer {
        usd_and_cents: u64,
    }

    #[test]
    pub fn transfer() {
        use crate::result::TxResult;

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
            .map(tim.clone(), |t, f| {
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
            .map(tim.clone(), |t, tim_funds| {
                Some(Funds {
                    sterling_and_pence: tim_funds.unwrap().sterling_and_pence,
                    usd_and_cents: tim_funds.unwrap().usd_and_cents - 100,
                })
            })
            .map(bob.clone(), |b, bob_funds| {
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
            .with_prerequisite(
                "Has available funds",
                [tim.clone()],
                |[tim_funds], transfer| {
                    tim_funds.is_some_and(|f| f.usd_and_cents >= transfer.usd_and_cents)
                },
            )
            .with_operation(tim.clone(), |tim_funds, transfer| {
                Some(Funds {
                    sterling_and_pence: tim_funds.unwrap().sterling_and_pence,
                    usd_and_cents: tim_funds.unwrap().usd_and_cents - transfer.usd_and_cents,
                })
            })
            .with_operation(bob.clone(), |bob_funds, transfer| {
                Some(bob_funds.map_or(
                    Funds {
                        usd_and_cents: transfer.usd_and_cents,
                        sterling_and_pence: 0,
                    },
                    |f| Funds {
                        usd_and_cents: f.usd_and_cents + transfer.usd_and_cents,
                        sterling_and_pence: f.sterling_and_pence,
                    },
                ))
            })
            .into_transaction();
        assert_eq!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 40 }),
            TxResult::Completed
        );
        assert_ne!(
            send_x_usd_from_bob_to_tim.execute(&Transfer { usd_and_cents: 20 }),
            TxResult::Completed
        );

        let add_100_usd_to_bob_if_exists = db
            .transaction()
            .modify(bob.clone(), |b, bob_funds| {
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
            .modify_or_insert_with(
                pam.clone(),
                |p, pam_funds| {
                    pam_funds.usd_and_cents += 123;
                },
                |p| Funds {
                    sterling_and_pence: 0,
                    usd_and_cents: 123,
                },
            )
            .into_transaction();
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(()));
        assert_eq!(add_123_to_pam.execute(), TxResult::Completed(()));
    }
}
// TODO use remove for modify
