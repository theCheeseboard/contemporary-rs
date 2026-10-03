//! Quote string modifier for locale-aware quotation marks.

use super::ModifierVariable;
use crate::{Locale, modifiers::StringModifier};
use std::fmt::Display;

/// A string modifier that wraps text in locale-appropriate quotation marks.
///
/// Different locales use different quotation mark styles:
///
/// | Locale       | Primary    | Alternate  |
/// |--------------|------------|------------|
/// | English (US) | "text"     | 'text'     |
/// | French       | « text »   | ‹ text ›   |
/// | German       | „text"     | ‚text'     |
/// | Japanese     | 「text」   | 『text』   |
///
/// # Usage in `tr!` Macro
///
/// ```rust,ignore
/// // Primary quotation marks (default)
/// tr!("SAID", "They said {{phrase}}.", phrase:quote = some_text);
/// // English: They said "hello".
/// // French: They said « hello ».
///
/// // Alternate (inner) quotation marks
/// tr!("NESTED", "She said {{outer}}.", outer:quote = format!("He said {}",
///     tr!("INNER", "{{inner}}", inner:quote("alt") = word)));
/// // English: She said "He said 'hello'".
/// // French: She said « He said ‹ hello › ».
/// ```
///
/// # Arguments
///
/// - No arguments: Use primary quotation marks
/// - `"alt"`: Use alternate (inner) quotation marks
///
/// # Example
///
/// ```rust,ignore
/// use cntp_localesupport::{Locale, modifiers::{Quote, StringModifier}};
///
/// let locale = Locale::new_from_locale_identifier("en-US");
/// let quote = Quote;
///
/// // Primary quotes
/// let result = quote.transform(&locale, "hello", &[]);
/// assert_eq!(result, "\"hello\"");
///
/// // Alternate quotes
/// let result = quote.transform(&locale, "hello", &[&(None, "alt")]);
/// assert_eq!(result, "'hello'");
/// ```
#[derive(Default)]
pub struct Quote;

impl<T> StringModifier<T> for Quote
where
    T: Display,
{
    /// Transform the input by wrapping it in quotation marks.
    ///
    /// If the first variable is `("alt")`, uses alternate quotation marks.
    /// Otherwise, uses primary quotation marks.
    fn transform<'a>(
        &self,
        locale: &Locale,
        input: T,
        variables: &'a [ModifierVariable<'a>],
    ) -> String {
        if let Some((None, "alt")) = variables.first() {
            locale.quote_string_alternate(input)
        } else {
            locale.quote_string(input)
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use crate::{
        Locale,
        modifiers::{Quote, StringModifier},
    };

    #[test]
    fn string_modifier() {
        let locale = Locale::new_from_locale_identifier("en-US");
        let modifier = Quote;
        let result = modifier.transform(&locale, "Hello", &[]);
        assert_eq!(result, "\"Hello\"");
    }

    #[test]
    fn alternate_string_modifier() {
        let locale = Locale::new_from_locale_identifier("en-US");
        let modifier = Quote;
        let result = modifier.transform(&locale, "Hello", &[&(None, "alt")]);
        assert_eq!(result, "'Hello'");
    }
}
