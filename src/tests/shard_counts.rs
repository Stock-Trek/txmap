#[cfg(test)]
mod tests {
    use crate::{new_types::ShardIndex, prelude::*};

    #[test]
    fn shard_indexes() {
        assert_eq!(u8::from(ShardCount::_8), 8);
        assert_eq!(u8::from(ShardCount::_16), 16);
        assert_eq!(u8::from(ShardCount::_32), 32);
        assert_eq!(u8::from(ShardCount::_64), 64);
        assert_eq!(u8::from(ShardCount::_128), 128);
    }

    #[test]
    fn bitmasks() {
        assert_eq!(ShardIndex(u8::from(ShardCount::_8) - 1).bitmask().0, 1 << 7);
        assert_eq!(
            ShardIndex(u8::from(ShardCount::_16) - 1).bitmask().0,
            1 << 15
        );
        assert_eq!(
            ShardIndex(u8::from(ShardCount::_32) - 1).bitmask().0,
            1 << 31
        );
        assert_eq!(
            ShardIndex(u8::from(ShardCount::_64) - 1).bitmask().0,
            1 << 63
        );
        assert_eq!(
            ShardIndex(u8::from(ShardCount::_128) - 1).bitmask().0,
            1 << 127
        );
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
