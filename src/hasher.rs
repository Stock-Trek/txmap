/// The default hasher used by [`TxMap`](crate::tx_map::TxMap).
///
/// When the `rapidhash` feature is enabled (default), this is
/// [`rapidhash::fast::RandomState`]; otherwise it falls back to
/// [`std::hash::RandomState`].
#[cfg(feature = "rapidhash")]
pub type DefaultBuildHasher = rapidhash::fast::RandomState;

#[cfg(not(feature = "rapidhash"))]
pub type DefaultBuildHasher = std::hash::RandomState;
