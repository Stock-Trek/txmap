use crate::{
    lock_policies::lock_policy::LockPolicy, result::TxResult, shards::Shards, tx_map::TxMap,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
};
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

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

impl<K, V, L, S> Serialize for TxMap<K, V, L, S>
where
    K: Hash + Eq + Serialize,
    V: Serialize,
    L: LockPolicy,
    S: BuildHasher,
{
    fn serialize<SER: Serializer>(&self, serializer: SER) -> Result<SER::Ok, SER::Error> {
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
        let txmap = TxMap::<K, V, L>::with_lock_policy(Shards::_16);
        while let Some((key, value)) = map.next_entry::<K, V>()? {
            txmap.insert(key, value);
        }
        Ok(txmap)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let txmap = TxMap::<K, V, L>::with_lock_policy(Shards::_16);
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
