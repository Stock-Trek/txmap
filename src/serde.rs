use crate::{
    lock_policies::lock_policy::LockPolicy, result::TxResult, shards::Shards, tx_map::TxMap,
    tx_map_builder::TxMapBuilder,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeMap,
};
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

impl<T: Serialize> Serialize for TxResult<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TxResult::Completed { state } => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("variant", "Completed")?;
                m.serialize_entry("state", state)?;
                m.end()
            }
            TxResult::RequirementNotMet {
                index,
                requirement,
                state,
            } => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("variant", "RequirementNotMet")?;
                m.serialize_entry("index", index)?;
                m.serialize_entry("requirement", requirement)?;
                m.serialize_entry("state", state)?;
                m.end()
            }
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for TxResult<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct TxResultHelper<T> {
            variant: String,
            index: Option<usize>,
            requirement: Option<String>,
            state: Option<T>,
        }

        let helper = TxResultHelper::<T>::deserialize(deserializer)?;
        match helper.variant.as_str() {
            "Completed" => {
                let state = helper
                    .state
                    .ok_or_else(|| de::Error::missing_field("state"))?;
                Ok(TxResult::Completed { state })
            }
            "RequirementNotMet" => {
                let index = helper
                    .index
                    .ok_or_else(|| de::Error::missing_field("index"))?;
                let requirement = helper
                    .requirement
                    .ok_or_else(|| de::Error::missing_field("requirement"))?;
                let state = helper
                    .state
                    .ok_or_else(|| de::Error::missing_field("state"))?;
                Ok(TxResult::RequirementNotMet {
                    index,
                    requirement,
                    state,
                })
            }
            other => Err(de::Error::unknown_variant(
                other,
                &["Completed", "RequirementNotMet"],
            )),
        }
    }
}

impl<K, V, L, S> Serialize for TxMap<K, V, L, S>
where
    K: Serialize,
    V: Serialize,
    L: LockPolicy,
    S: BuildHasher,
{
    fn serialize<SER: Serializer>(&self, serializer: SER) -> Result<SER::Ok, SER::Error> {
        let entries: Vec<(&K, &V)> = self.iter().collect();
        (self.shard_count.0, entries).serialize(serializer)
    }
}

fn shard_count_to_shards(count: u8) -> Result<Shards, String> {
    match count {
        8 => Ok(Shards::_8),
        16 => Ok(Shards::_16),
        32 => Ok(Shards::_32),
        64 => Ok(Shards::_64),
        128 => Ok(Shards::_128),
        other => Err(format!("invalid shard count: {other}")),
    }
}

struct TxMapVisitor<K, V, L> {
    _marker: PhantomData<(K, V, L)>,
}

impl<'de, K, V, L> Visitor<'de> for TxMapVisitor<K, V, L>
where
    K: Hash + Eq + Deserialize<'de>,
    V: Deserialize<'de>,
    L: LockPolicy,
{
    type Value = TxMap<K, V, L>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a shard count followed by a sequence of key-value pairs")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let shard_count: u8 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let shards = shard_count_to_shards(shard_count).map_err(de::Error::custom)?;
        let txmap = TxMapBuilder::default()
            .with_lock_policy::<L>()
            .with_shards(shards)
            .build();
        let entries: Option<Vec<(K, V)>> = seq.next_element()?;
        if let Some(entries) = entries {
            for (key, value) in entries {
                txmap.insert(key, value);
            }
        }
        Ok(txmap)
    }
}

impl<'de, K, V, L> Deserialize<'de> for TxMap<K, V, L>
where
    K: Hash + Eq + Deserialize<'de>,
    V: Deserialize<'de>,
    L: LockPolicy,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(TxMapVisitor::<K, V, L> {
            _marker: PhantomData,
        })
    }
}
