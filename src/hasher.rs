#[cfg(feature = "rapidhash")]
pub type DefaultBuildHasher = rapidhash::fast::RandomState;

#[cfg(not(feature = "rapidhash"))]
pub type DefaultBuildHasher = std::hash::RandomState;
