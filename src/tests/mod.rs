pub mod creators;
pub mod data;
pub mod types;

#[cfg(feature = "rayon")]
pub mod rayon;
#[cfg(feature = "serde")]
pub mod serde;

pub mod prepared {
    pub mod get_op;
    pub mod get_or_insert_op;
    pub mod guards;
    pub mod insert_with_if_absent_op;
    pub mod insert_with_op;
    pub mod last_used_key;
    pub mod modify_op;
    pub mod move_value_op;
    pub mod remove_if_op;
    pub mod remove_op;
    pub mod swap_value_op;
    pub mod transaction;
    pub mod transfer;
    pub mod update_op;
}

pub mod immediate {
    pub mod get_op;
    pub mod get_or_insert_op;
    pub mod guards;
    pub mod insert_with_if_absent_op;
    pub mod insert_with_op;
    pub mod modify_op;
    pub mod move_value_op;
    pub mod remove_if_op;
    pub mod remove_op;
    pub mod swap_value_op;
    pub mod transaction;
    pub mod update_op;
}

pub mod map {
    pub mod api_surface;
    pub mod basic_operations;
    pub mod concurrency;
    pub mod get_or_insert;
    pub mod global;
    pub mod indexer;
    pub mod insert;
    pub mod iterator;
    pub mod shard_counts;
}
