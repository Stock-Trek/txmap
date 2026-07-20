#[cfg(test)]
pub(crate) mod types {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub(crate) struct User {
        first_name: String,
        last_name: String,
    }
    #[derive(Debug, Default)]
    pub(crate) struct Funds {
        usd_and_cents: u64,
        sterling_and_pence: u64,
    }
    #[derive(Debug, Default)]
    pub(crate) struct Transfer {
        usd_and_cents: u64,
    }

    #[derive(Debug, Default, Clone, PartialEq)]
    pub(crate) struct Counter {
        value: u64,
    }
}
