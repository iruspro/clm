//! Monetary primitives: [`Money`] (an amount in a currency) and [`Currency`].
//!
//! Amounts are stored as integer **minor units** (e.g. cents), never as floats,
//! so that arithmetic is exact and totals always balance.

mod error;

use std::fmt::{self, Display, Formatter, Write as _};
use std::ops::Neg;

pub use self::error::MoneyError;

/// Separates groups of three digits in the whole part of an amount.
const GROUP_SEPARATOR: char = ',';

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

    /// Creates a zero amount in the given `currency`.
    ///
    /// Handy as the starting value when summing amounts of one currency.
    pub fn zero(currency: Currency) -> Self {
        Money {
            amount: 0,
            currency,
        }
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
    pub fn checked_add(self, rhs: Money) -> Result<Money, MoneyError> {
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
    pub fn checked_sub(self, rhs: Money) -> Result<Money, MoneyError> {
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
    pub fn try_sum(items: impl IntoIterator<Item = Money>) -> Result<Option<Money>, MoneyError> {
        let mut iter = items.into_iter();
        let Some(first) = iter.next() else {
            return Ok(None);
        };

        iter.try_fold(first, Money::checked_add).map(Some)
    }

    /// Returns `true` if the amount is greater than zero.
    pub fn is_positive(self) -> bool {
        self.amount > 0
    }

    /// Returns `true` if the amount is less than zero.
    pub fn is_negative(self) -> bool {
        self.amount < 0
    }

    /// Returns `true` if the amount is exactly zero.
    pub fn is_zero(self) -> bool {
        self.amount == 0
    }
}

/// Writes the amount for a human: the currency's symbol, the whole part in
/// groups of three, and one digit per decimal the currency has.
///
/// Honours the width and alignment of the format spec, so `{:>12}` lines
/// amounts up in a column.
impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let decimals = usize::from(self.currency.decimals());

        // Split the *magnitude*, not the signed amount: `-5` cents divided by
        // 100 is `0` with a remainder of `-5`, which would print as `0.-5`.
        // `unsigned_abs` rather than `abs` because `i64::MIN.abs()` overflows —
        // it has no positive counterpart. The sign is re-applied by
        // `pad_integral` below.
        let magnitude = self.amount.unsigned_abs();
        let scale = 10u64.pow(self.currency.decimals().into());
        let (whole, fraction) = (magnitude / scale, magnitude % scale);

        let mut buf = String::from(self.currency.symbol());
        write_grouped(&mut buf, whole);
        if decimals > 0 {
            // `{:0decimals$}` pads with leading zeros to the currency's scale,
            // so 5 cents is `.05` and one satoshi is `.00000001`.
            write!(buf, ".{fraction:0decimals$}")?;
        }

        // `pad_integral` is the std hook for numeric `Display` impls: given the
        // sign and the unsigned digits, it applies the `+` flag and honours the
        // width and alignment a plain `write!` would ignore.
        f.pad_integral(!self.is_negative(), "", &buf)
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
///
/// The discriminants are part of the storage format (see
/// [`as_u16`](Currency::as_u16) / [`TryFrom<u16>`](Currency::try_from)), so they
/// are written out explicitly and must never be renumbered.
// ISO 4217 codes are conventionally uppercase; keep them as the variant names.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    EUR = 0,
    USD = 1,
    RUB = 2,
    UAH = 3,
    BTC = 4,
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

    /// The symbol [`Money`] writes in front of an amount.
    ///
    /// The match is exhaustive on purpose: a currency added above fails to
    /// compile here until it is given a symbol.
    pub fn symbol(self) -> &'static str {
        match self {
            Currency::EUR => "€",
            Currency::USD => "$",
            Currency::RUB => "₽",
            Currency::UAH => "₴",
            Currency::BTC => "₿",
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

    /// Encodes the currency as its stable discriminant, for storage.
    ///
    /// The inverse is [`TryFrom<u16>`](Currency::try_from).
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Decodes a currency from its stored discriminant, the inverse of
/// [`as_u16`](Currency::as_u16).
///
/// Returns [`MoneyError::UnknownCurrency`] for a value no variant claims.
impl TryFrom<u16> for Currency {
    type Error = MoneyError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Currency::ALL
            .into_iter()
            .find(|currency| currency.as_u16() == value)
            .ok_or(MoneyError::UnknownCurrency(value))
    }
}

/// Appends `whole` to `out`, separating every three digits from the right:
/// `1234567` becomes `1,234,567`.
fn write_grouped(out: &mut String, whole: u64) {
    let digits = whole.to_string();
    for (i, digit) in digits.chars().enumerate() {
        // Digits are ASCII, so the char index and the byte length are
        // comparable: `digits.len() - i` is how many digits are still to come.
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(GROUP_SEPARATOR);
        }
        out.push(digit);
    }
}
