#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct User {
    pub first_name: String,
    pub last_name: String,
}
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct Funds {
    pub usd_and_cents: u64,
    pub sterling_and_pence: u64,
}
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct Transfer {
    pub usd_and_cents: u64,
}

#[cfg(test)]
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Counter {
    pub value: u64,
}
