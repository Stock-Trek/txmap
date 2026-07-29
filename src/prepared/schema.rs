use crate::new_types::ShardCount;
use std::hash::{BuildHasher, Hash};

pub trait TxSchema<K>
where
    K: Hash + Eq,
{
    type Keys: TxKeys<K, Self::IndexedKeys>;
    type IndexedKeys;
    type Params;
    type State: Default;
}
pub trait TxKeys<K, IndexedKeys>
where
    K: Hash + Eq,
{
    fn into_indexed<S: BuildHasher>(self, shard_count: ShardCount, hash_builder: &S)
    -> IndexedKeys;
}

pub trait TxKeySelector<K, KEYS>
where
    K: Hash + Eq,
{
    fn get<'keys>(&self, keys: &'keys KEYS) -> &'keys K;
}

#[macro_export]
macro_rules! tx_schema {
    (
        $name:ident,
        keys: [ $($key:ident),* $(,)? ],
        params: { $($param_field:ident: $param_type:ty),* $(,)? },
        state: { $($state_field:ident: $state_type:ty),* $(,)? }
        $(,)?
    ) => {
        $crate::_paste! {
            pub use [<__private $name>] :: $name;
            pub use [<__private $name>] :: [<$name Keys>];
            pub use [<__private $name>] :: [<$name Params>];
            pub use [<__private $name>] :: [<$name State>];
            #[allow(non_snake_case)]
            mod [<__private $name>] {
                // schema
                pub struct $name <K>
                where
                    K: std::hash::Hash + Eq,
                {
                    _phantom: std::marker::PhantomData<K>,
                }
                impl<K> $crate::prelude::TxSchema<K> for $name<K>
                where
                    K: std::hash::Hash + Eq,
                {
                    type Keys =   [<$name Keys>]<K>;
                    type IndexedKeys = [<$name IndexedKeys>]<K>;
                    type Params = [<$name Params>];
                    type State =  [<$name State>];
                }
                impl<K> $name <K>
                where
                    K: std::hash::Hash + Eq,
                {
                    pub const SCHEMA: $name <K> = $name {
                        _phantom: std::marker::PhantomData,
                    };
                    $(
                        #[allow(non_upper_case_globals)]
                        pub const $key: [<$name _ $key>]<K> = [<$name _ $key>] {
                            _phantom: std::marker::PhantomData,
                        };
                    )*
                }

                // keys
                pub struct [<$name Keys>]<K>
                where
                    K: std::hash::Hash + Eq,
                {
                    $(pub $key: K,)*
                }

                // params
                pub struct [<$name Params>] {
                    $(pub $param_field: $param_type,)*
                }

                // state
                #[derive(Debug, Default, Hash, PartialEq, Eq)]
                pub struct [<$name State>] {
                    $(pub $state_field: $state_type,)*
                }

                pub struct [<$name IndexedKeys>]<K>
                where
                    K: std::hash::Hash + Eq,
                {
                    $(pub $key: $crate::prelude::TxKey<K>,)*
                }
                $(
                    #[allow(non_camel_case_types)]
                    pub struct [<$name _ $key>]<K>
                    where
                        K: std::hash::Hash + Eq,
                    {
                    _phantom: std::marker::PhantomData<K>,
                    }
                )*
                $(
                    impl<K> $crate::prelude::TxKeySelector<$crate::prelude::TxKey<K>, [<$name IndexedKeys>]<K>> for [<$name _ $key>]<K>
                    where
                        K: std::hash::Hash + Eq,
                    {
                        fn get<'keys>(&self, keys: &'keys [<$name IndexedKeys>]<K>) -> &'keys $crate::prelude::TxKey<K> {
                            &keys.$key
                        }
                    }
                )*
                impl<K> $crate::prelude::TxKeys<K, [<$name IndexedKeys>]<K>> for [<$name Keys>]<K>
                where
                    K: std::hash::Hash + Eq,
                {
                    fn into_indexed<S: std::hash::BuildHasher>(self, shard_count: $crate::prelude::ShardCount, hash_builder: &S) -> [<$name IndexedKeys>]<K> {
                        [<$name IndexedKeys>] {
                            $(
                                $key: $crate::prelude::Indexer::indexed_key(shard_count, self.$key, hash_builder),
                            )*
                        }
                    }
                }
            }
        }
    };
}

pub use tx_schema;
