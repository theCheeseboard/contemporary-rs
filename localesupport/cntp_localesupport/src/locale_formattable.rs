//! Locale-aware value formatting.
//!
//! This module provides the [`LocaleFormattable`] trait for converting values to
//! locale-appropriate string representations. It is used internally by the `tr!`
//! macro to format variable values before substitution.
//!
//! ## Built-in implementations
//!
//! The trait is implemented for:
//!
//! - All integer types (`i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`)
//! - Floating point types (`f32`, `f64`)
//! - Strings (`str`, `String`)
//!
//! ## Example
//!
//! ```rust
//! use cntp_localesupport::Locale;
//! use cntp_localesupport::locale_formattable::LocaleFormattable;
//!
//! let us_locale = Locale::new_from_locale_identifier("en-US");
//! let de_locale = Locale::new_from_locale_identifier("de-DE");
//!
//! let number: i32 = 1234567;
//!
//! // US English: "1,234,567"
//! let us_formatted = number.to_locale_string(&us_locale);
//!
//! // German: "1.234.567"
//! let de_formatted = number.to_locale_string(&de_locale);
//! ```

use crate::Locale;
use crate::list_builder::ListBuilder;
use icu::decimal::input::Decimal;
use std::str::FromStr;

/// A trait for values that can be formatted according to locale conventions.
///
/// This trait is the foundation of locale-aware formatting in the Contemporary i18n
/// system. It provides a single method, [`to_locale_string`](LocaleFormattable::to_locale_string),
/// that converts a value to its locale-appropriate string representation.
///
/// # Implementing for Custom Types
///
/// You can implement this trait for your own types:
///
/// ```rust,ignore
/// use cntp_localesupport::{Locale, locale_formattable::LocaleFormattable};
///
/// struct Money {
///     amount: i64,  // in cents
///     currency: String,
/// }
///
/// impl LocaleFormattable for Money {
///     fn to_locale_string(&self, locale: &Locale) -> String {
///         let whole = self.amount / 100;
///         let cents = (self.amount % 100).abs();
///         // Use locale's number formatting for the amount
///         format!("{}.{:02} {}", whole.to_locale_string(locale), cents, self.currency)
///     }
/// }
/// ```
///
/// # Usage with `tr!`
///
/// When you use a variable in the `tr!` macro without a modifier, and the value
/// implements `LocaleFormattable`, it will be automatically formatted:
///
/// ```rust,ignore
/// let count: i32 = 1000000;
/// tr!("BIG_NUMBER", "That's {{n}} items!", n = count);
/// // US English: "That's 1,000,000 items!"
/// // German: "That's 1.000.000 items!"
/// ```
///
/// To disable automatic locale formatting, prefix the value with `!`:
///
/// ```rust,ignore
/// tr!("RAW_NUMBER", "Value: {{n}}", n = !count);
/// // All locales: "Value: 1000000"
/// ```
pub trait LocaleFormattable {
    /// Convert this value to a locale-formatted string.
    ///
    /// # Arguments
    ///
    /// * `locale` - The locale to use for formatting
    ///
    /// # Returns
    ///
    /// A string representation of the value formatted according to the locale's conventions.
    fn to_locale_string(&self, locale: &Locale) -> String;
}

macro_rules! locale_formattable_integer_impl {
    ($typ:ty) => {
        impl LocaleFormattable for $typ {
            fn to_locale_string(&self, locale: &Locale) -> String {
                locale.format_decimal(Decimal::from(*self))
            }
        }
    };
}

locale_formattable_integer_impl!(i8);
locale_formattable_integer_impl!(i16);
locale_formattable_integer_impl!(i32);
locale_formattable_integer_impl!(i64);
locale_formattable_integer_impl!(i128);
locale_formattable_integer_impl!(isize);
locale_formattable_integer_impl!(u8);
locale_formattable_integer_impl!(u16);
locale_formattable_integer_impl!(u32);
locale_formattable_integer_impl!(u64);
locale_formattable_integer_impl!(u128);
locale_formattable_integer_impl!(usize);

macro_rules! locale_formattable_stringable_impl {
    ($typ:ty) => {
        impl LocaleFormattable for $typ {
            fn to_locale_string(&self, locale: &Locale) -> String {
                locale.format_decimal(Decimal::from_str(self.to_string().as_str()).unwrap())
            }
        }
    };
}

locale_formattable_stringable_impl!(f32);
locale_formattable_stringable_impl!(f64);

impl LocaleFormattable for str {
    fn to_locale_string(&self, _: &Locale) -> String {
        self.to_string()
    }
}

impl LocaleFormattable for String {
    fn to_locale_string(&self, locale: &Locale) -> String {
        self.as_str().to_locale_string(locale)
    }
}

impl LocaleFormattable for ListBuilder<'_, '_> {
    fn to_locale_string(&self, _: &Locale) -> String {
        self.build()
    }
}
