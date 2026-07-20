#[cfg(test)]
mod shard_counts {
    use crate::{prelude::*, shard_count::all_guards_bitmask};

    #[test]
    fn bitmasks() {
        assert_eq!(all_guards_bitmask(u8::from(ShardCount::_8)), !0u8 as u128);
        assert_eq!(all_guards_bitmask(u8::from(ShardCount::_16)), !0u16 as u128);
        assert_eq!(all_guards_bitmask(u8::from(ShardCount::_32)), !0u32 as u128);
        assert_eq!(all_guards_bitmask(u8::from(ShardCount::_64)), !0u64 as u128);
        assert_eq!(all_guards_bitmask(u8::from(ShardCount::_128)), !0u128);
    }

    #[test]
    fn sanity_check() {
        for sc in [
            ShardCount::_8,
            ShardCount::_16,
            ShardCount::_32,
            ShardCount::_64,
            ShardCount::_128,
        ] {
            let map: TxMap<u64, u64> = TxMap::new(sc);
            assert!(map.is_empty());
            map.insert(1, 10);
            assert_eq!(map.len(), 1);
            assert_eq!(map.get_with(&1, |v| *v), Some(10));
        }
    }
}
