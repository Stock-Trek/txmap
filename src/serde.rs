use crate::{
    lock_policies::lock_policy::LockPolicy,
    new_types::{BitMask, HashCode, ShardCount, ShardIndex},
    result::TxResult,
    shards::Shards,
    tx_map::TxMap,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
};
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

// ---------------------------------------------------------------------------
// HashCode – transparent u64
// ---------------------------------------------------------------------------

impl Serialize for HashCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HashCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u64::deserialize(deserializer).map(HashCode)
    }
}

// ---------------------------------------------------------------------------
// ShardCount – transparent u8
// ---------------------------------------------------------------------------

impl Serialize for ShardCount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ShardCount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer).map(ShardCount)
    }
}

// ---------------------------------------------------------------------------
// ShardIndex – transparent u8
// ---------------------------------------------------------------------------

impl Serialize for ShardIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ShardIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u8::deserialize(deserializer).map(ShardIndex)
    }
}

// ---------------------------------------------------------------------------
// BitMask – transparent u128
// ---------------------------------------------------------------------------

impl Serialize for BitMask {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BitMask {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u128::deserialize(deserializer).map(BitMask)
    }
}

// ---------------------------------------------------------------------------
// Shards – enum with unit variants
// ---------------------------------------------------------------------------

impl Serialize for Shards {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Shards::_8 => serializer.serialize_unit_variant("Shards", 0, "_8"),
            Shards::_16 => serializer.serialize_unit_variant("Shards", 1, "_16"),
            Shards::_32 => serializer.serialize_unit_variant("Shards", 2, "_32"),
            Shards::_64 => serializer.serialize_unit_variant("Shards", 3, "_64"),
            Shards::_128 => serializer.serialize_unit_variant("Shards", 4, "_128"),
        }
    }
}

impl<'de> Deserialize<'de> for Shards {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ShardsVisitor;

        impl<'de> Visitor<'de> for ShardsVisitor {
            type Value = Shards;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("one of: _8, _16, _32, _64, _128")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Shards, E> {
                match value {
                    "_8" => Ok(Shards::_8),
                    "_16" => Ok(Shards::_16),
                    "_32" => Ok(Shards::_32),
                    "_64" => Ok(Shards::_64),
                    "_128" => Ok(Shards::_128),
                    other => Err(de::Error::unknown_variant(
                        other,
                        &["_8", "_16", "_32", "_64", "_128"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(ShardsVisitor)
    }
}

// ---------------------------------------------------------------------------
// TxResult<T>
// ---------------------------------------------------------------------------

impl<T: Serialize> Serialize for TxResult<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TxResult::Completed(state) => {
                let mut m = serializer.serialize_map(Some(2))?;
                m.serialize_entry("status", "Completed")?;
                m.serialize_entry("value", state)?;
                m.end()
            }
            TxResult::RequirementNotMet(index, name, state) => {
                let mut m = serializer.serialize_map(Some(4))?;
                m.serialize_entry("status", "RequirementNotMet")?;
                m.serialize_entry("index", index)?;
                m.serialize_entry("name", name)?;
                m.serialize_entry("value", state)?;
                m.end()
            }
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for TxResult<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct TxResultHelper<T> {
            status: String,
            index: Option<usize>,
            name: Option<String>,
            value: Option<T>,
        }

        let helper = TxResultHelper::<T>::deserialize(deserializer)?;
        match helper.status.as_str() {
            "Completed" => {
                let value = helper
                    .value
                    .ok_or_else(|| de::Error::missing_field("value"))?;
                Ok(TxResult::Completed(value))
            }
            "RequirementNotMet" => {
                let index = helper
                    .index
                    .ok_or_else(|| de::Error::missing_field("index"))?;
                let name = helper
                    .name
                    .ok_or_else(|| de::Error::missing_field("name"))?;
                let value = helper
                    .value
                    .ok_or_else(|| de::Error::missing_field("value"))?;
                Ok(TxResult::RequirementNotMet(index, name, value))
            }
            other => Err(de::Error::unknown_variant(
                other,
                &["Completed", "RequirementNotMet"],
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// TxMap<K, V, L>
//
// Serialised as a map of key → value entries.
// ---------------------------------------------------------------------------

impl<K, V, L> Serialize for TxMap<K, V, L>
where
    K: Hash + Eq + Serialize,
    V: Serialize,
    L: LockPolicy,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.len();
        let mut map = serializer.serialize_map(Some(len))?;
        for guard in self.custodian.all_read_guards() {
            for (k, v) in guard.1.iter() {
                map.serialize_entry(k, v)?;
            }
        }
        map.end()
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
        f.write_str("a map of key-value pairs")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let txmap = TxMap::<K, V>::with_lock_policy::<L>(Shards::_16);
        while let Some((key, value)) = map.next_entry::<K, V>()? {
            txmap.insert(key, value);
        }
        Ok(txmap)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let txmap = TxMap::<K, V>::with_lock_policy::<L>(Shards::_16);
        while let Some((key, value)) = seq.next_element::<(K, V)>()? {
            txmap.insert(key, value);
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
        deserializer.deserialize_map(TxMapVisitor::<K, V, L> {
            _marker: PhantomData,
        })
    }
}
