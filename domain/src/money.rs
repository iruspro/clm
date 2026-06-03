//! Monetary primitives: [`Money`] (an amount in a currency) and [`Currency`].
//!
//! Amounts are stored as integer **minor units** (e.g. cents), never as floats,
//! so that arithmetic is exact and totals always balance.

use std::ops::Neg;

pub mod error;
pub use error::{MoneyError, MoneyResult};

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

    /// Adds two amounts. Fails on currency mismatch or `i64` overflow.
    pub fn checked_add(self, rhs: Money) -> MoneyResult<Money> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: rhs.currency,
            });
        }
        let amount = self
            .amount
            .checked_add(rhs.amount)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency))
    }

    /// Subtracts `rhs` from `self`. Fails on currency mismatch or `i64` overflow.
    pub fn checked_sub(self, rhs: Money) -> MoneyResult<Money> {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: rhs.currency,
            });
        }
        let amount = self
            .amount
            .checked_sub(rhs.amount)
            .ok_or(MoneyError::Overflow)?;
        Ok(Money::new(amount, self.currency))
    }

    /// Sums an iterator of amounts, failing on currency mismatch or overflow.
    ///
    /// Returns `Ok(None)` for an empty iterator — an empty sum has no currency,
    /// so there is no meaningful zero to return. All items must share one currency.
    pub fn try_sum(items: impl IntoIterator<Item = Money>) -> MoneyResult<Option<Money>> {
        let mut iter = items.into_iter();
        let Some(first) = iter.next() else {
            return Ok(None);
        };

        iter.try_fold(first, Money::checked_add).map(Some)
    }
}

/// Negates the amount, keeping the currency. Used to flip a posting between
/// debit and credit. Infallible in practice (overflows only at `i64::MIN`,
/// which is unreachable for real amounts), so it's an operator rather than a
/// `checked_*` method.
impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        Money::new(-self.amount, self.currency)
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
