//! Monetary primitives: [`Money`] (an amount in a currency) and [`Currency`].
//!
//! Amounts are stored as integer **minor units** (e.g. cents), never as floats,
//! so that arithmetic is exact and totals always balance.

/// A monetary amount in a specific currency.
///
/// The amount is held in the currency's **minor unit** (the smallest
/// indivisible unit). For example, the minor unit of USD is the cent, so
/// `Money::new(150, Currency::USD)` represents $1.50.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money {
    /// Amount in minor units (e.g. cents for USD, satoshis for BTC).
    amount: i64,
    currency: Currency,
}

impl Money {
    /// Creates a `Money` value from an `amount` in minor units and its `currency`.
    pub fn new(amount: i64, currency: Currency) -> Self {
        Money { amount, currency }
    }

    /// Returns the amount in minor units (e.g. `150` for $1.50).
    pub fn amount(self) -> i64 {
        self.amount
    }

    /// Returns the currency this amount is denominated in.
    pub fn currency(self) -> Currency {
        self.currency
    }
}

/// A supported currency.
///
/// The currency determines how many decimal places an amount has — see
/// [`Currency::decimals`].
// ISO 4217 codes are conventionally uppercase; keep them as the variant names.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    EUR,
    USD,
    RUB,
    UAH,
    BTC,
}

impl Currency {
    /// All supported currencies — the single source of truth for the variant list.
    pub const ALL: [Currency; 5] = [
        Currency::EUR,
        Currency::USD,
        Currency::RUB,
        Currency::UAH,
        Currency::BTC,
    ];

    /// Number of decimal places in this currency's minor unit.
    ///
    /// For example, USD has 2 (1 dollar = 100 cents) and BTC has 8
    /// (1 bitcoin = 100,000,000 satoshis).
    pub fn decimals(self) -> u8 {
        match self {
            Currency::EUR | Currency::USD | Currency::RUB | Currency::UAH => 2,
            Currency::BTC => 8,
        }
    }

    /// The currency's standard short code (ISO 4217 for fiat, common ticker for crypto).
    pub fn code(self) -> &'static str {
        match self {
            Currency::EUR => "EUR",
            Currency::USD => "USD",
            Currency::RUB => "RUB",
            Currency::UAH => "UAH",
            Currency::BTC => "BTC",
        }
    }

    /// Parses a currency from its [`code`](Currency::code), ignoring case
    /// (`"usd"`, `"USD"`, `"Usd"` all work). Returns `None` for an unknown code.
    pub fn from_code(code: &str) -> Option<Currency> {
        Currency::ALL
            .into_iter()
            .find(|c| c.code().eq_ignore_ascii_case(code))
    }
}
