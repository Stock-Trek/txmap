#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct User {
    pub(crate) first_name: String,
    pub(crate) last_name: String,
}
#[derive(Debug, Default)]
pub(crate) struct Funds {
    pub(crate) usd_and_cents: u64,
    pub(crate) sterling_and_pence: u64,
}
#[derive(Debug, Default)]
pub(crate) struct Transfer {
    pub(crate) usd_and_cents: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Counter {
    pub(crate) value: u64,
}
