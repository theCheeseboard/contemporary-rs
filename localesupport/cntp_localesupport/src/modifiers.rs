//! String modifiers for locale-aware transformations.
//!
//! This module provides the [`StringModifier`] trait and built-in modifier implementations
//! that transform values according to locale conventions. Modifiers are typically used
//! with the `tr!` macro from `cntp_i18n`.
//!
//! ## Built-in modifiers
//!
//! - [`Date`] - Format dates and times according to locale conventions
//! - [`Quote`] - Wrap strings in locale-appropriate quotation marks
//!
//! ## Implementing custom modifiers
//!
//! You can create custom modifiers by implementing the [`StringModifier`] trait:
//!
//! ```rust,ignore
//! use cntp_localesupport::{Locale, modifiers::{StringModifier, ModifierVariable}};
//!
//! struct Uppercase;
//!
//! impl<T: AsRef<str>> StringModifier<T> for Uppercase {
//!     fn transform<'a>(
//!         &self,
//!         _locale: &Locale,
//!         input: T,
//!         _variables: &'a [ModifierVariable<'a>],
//!     ) -> String {
//!         input.as_ref().to_uppercase()
//!     }
//! }
//! ```

mod date;
mod fixed;
mod quote;

pub use date::Date;
pub use fixed::Fixed;
pub use quote::Quote;

use crate::Locale;

/// A variable argument passed to a modifier.
///
/// Each variable is a tuple of an optional name and a string value.
/// Named variables use syntax like `name = "value"`, while positional
/// variables are just `"value"`.
///
/// # Example
///
/// For `Date(format = "YMD", length = "short")`:
/// - `(Some("format"), "YMD")`
/// - `(Some("length"), "short")`
///
/// For `Date("YMD")`:
/// - `(None, "YMD")`
pub type ModifierVariable<'a> = &'a (Option<&'a str>, &'a str);

/// A trait for locale-aware string transformations.
///
/// Modifiers transform input values into locale-appropriate string representations.
/// They can accept configuration through variables passed in the modifier invocation.
///
/// # Type parameter
///
/// The type parameter `T` represents the input type that this modifier can transform.
///
/// # Example implementation
///
/// ```rust,ignore
/// use cntp_localesupport::{Locale, modifiers::{StringModifier, ModifierVariable}};
///
/// /// A modifier that wraps text in brackets.
/// struct Bracket;
///
/// impl StringModifier<&str> for Bracket {
///     fn transform<'a>(
///         &self,
///         _locale: &Locale,
///         input: &str,
///         variables: &'a [ModifierVariable<'a>],
///     ) -> String {
///         // Check for "style" variable to customize brackets
///         let (open, close) = variables
///             .iter()
///             .find(|(name, _)| *name == Some("style"))
///             .map(|(_, val)| match *val {
///                 "square" => ("[", "]"),
///                 "curly" => ("{", "}"),
///                 "angle" => ("<", ">"),
///                 _ => ("(", ")"),
///             })
///             .unwrap_or(("(", ")"));
///
///         format!("{open}{input}{close}")
///     }
/// }
/// ```
pub trait StringModifier<T> {
    /// Transform the input value into a locale-appropriate string.
    ///
    /// # Arguments
    ///
    /// * `locale` - The current locale for formatting decisions
    /// * `input` - The value to transform
    /// * `variables` - Configuration variables from the modifier invocation
    ///
    /// # Returns
    ///
    /// The transformed string representation of the input.
    fn transform<'a>(
        &self,
        locale: &Locale,
        input: T,
        variables: &'a [ModifierVariable<'a>],
    ) -> String;
}
