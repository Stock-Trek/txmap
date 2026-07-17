use crate::{
    custodian::Custodian, indexer::Indexer, shard_count::ShardCount, tx_stem_builder::TxStemBuilder,
};
use std::hash::{DefaultHasher, Hash};

pub struct TxMap<K, V>
where
    K: Hash + Eq,
{
    indexer: Indexer,
    owned_key: fn(&K) -> K,
    custodian: Custodian<K, V>,
}

impl<K, V> TxMap<K, V>
where
    K: Clone + Hash + Eq,
{
    pub fn with_cloneable_key(shard_count: ShardCount) -> Self {
        Self::new(shard_count, |k| k.clone())
    }
}

impl<K, V> TxMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new(shard_count: ShardCount, owned_key: fn(&K) -> K) -> Self {
        let indexer = Indexer {
            shard_count: u8::from(shard_count) as u64,
            hasher_creator: || Box::new(DefaultHasher::new()),
        };
        Self {
            indexer,
            owned_key,
            custodian: Custodian::new(shard_count),
        }
    }
    pub fn transaction<'txmap>(&'txmap self) -> TxStemBuilder<'txmap, K, V> {
        TxStemBuilder {
            indexer: self.indexer,
            owned_key: self.owned_key,
            custodian: &self.custodian,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct User {
        first_name: String,
        last_name: String,
    }
    struct Funds {
        usd_and_cents: u64,
        sterling_and_pence: u64,
    }

    #[test]
    pub fn transfer() {
        use crate::result::TxResult;

        let db = TxMap::<User, Funds>::with_cloneable_key(ShardCount::_8);
        let bob = User {
            first_name: "Bob".into(),
            last_name: "Bobson".into(),
        };
        let tim = User {
            first_name: "Tim".into(),
            last_name: "Timson".into(),
        };
        db.transaction()
            .with_operation(tim.clone(), |_| {
                Some(Funds {
                    usd_and_cents: 150,
                    sterling_and_pence: 0,
                })
            })
            .build()
            .execute();
        let send_1_usd_from_bob_to_tim = db
            .transaction()
            .with_prerequisite("Has available funds", [tim.clone()], |[tim_funds]| {
                tim_funds.is_some_and(|f| f.usd_and_cents > 100)
            })
            .with_operation(tim, |tim_funds| {
                Some(Funds {
                    sterling_and_pence: tim_funds.unwrap().sterling_and_pence,
                    usd_and_cents: tim_funds.unwrap().usd_and_cents - 100,
                })
            })
            .with_operation(bob, |bob_funds| {
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
            .build();
        assert_eq!(send_1_usd_from_bob_to_tim.execute(), TxResult::Completed);
        assert_ne!(send_1_usd_from_bob_to_tim.execute(), TxResult::Completed);
    }
}
