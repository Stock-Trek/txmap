mod private {
    pub trait Sealed {}
    impl Sealed for super::PreparedBuilderPhase {}
    impl Sealed for super::PreparedBuildablePhase {}
}

pub trait BuildPhase: private::Sealed {}
impl BuildPhase for PreparedBuilderPhase {}
impl BuildPhase for PreparedBuildablePhase {}

/// Transaction is still accepting guard requirements.
pub struct PreparedBuilderPhase;
/// Transaction has at least one operation and can be built.
pub struct PreparedBuildablePhase;
