use crate::Locale;
use crate::modifiers::{ModifierVariable, StringModifier};
use icu::decimal::input::Decimal;

/// A string modifier that formats a number as a fixed-point decimal.
///
/// This macro allows you to select the number of decimal places to
/// render a number to, following the rules of the locale.
/// Here the results for the number 123456.789:
///
/// | Argument     | Result        |
/// |--------------|---------------|
/// | 3            | 123456.789    |
/// | 5            | 123456.78900  |
/// | 0            | 123457        |
/// | -2           | 123400        |
///
/// # Usage in `tr!` Macro
///
/// ```rust,ignore
/// tr!("LEFT_TURN", "Turn left in {{distance}} kilometers", distance:fixed = 5.18);
/// // Turn left in 5 kilometers.
///
/// tr!("LEFT_TURN", "Turn left in {{distance}} kilometers", distance:fixed("1") = 5.18);
/// // Turn left in 5.2 kilometers.
/// ```
///
/// # Arguments
///
/// - No arguments: Fix at distance 0
/// - `"(number)"`: Fix at the specified distance
pub struct Fixed;

impl StringModifier<&str> for Fixed {
    fn transform<'a>(
        &self,
        locale: &Locale,
        input: &str,
        variables: &'a [ModifierVariable<'a>],
    ) -> String {
        let fixed_length = -variables
            .first()
            .map(|(_, value)| value.parse::<i16>().expect("Invalid fixed length"))
            .unwrap_or(0);
        let mut decimal = Decimal::try_from_str(input).unwrap();
        decimal.absolute.round(fixed_length);
        locale.format_decimal(decimal)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use crate::{
        Locale,
        modifiers::{Fixed, StringModifier},
    };

    #[test]
    fn fixed_default() {
        let locale = Locale::new_from_locale_identifier("en-US");
        let modifier = Fixed;
        let result = modifier.transform(&locale, &&&*std::f32::consts::PI.to_string(), &[]);
        assert_eq!(result, format!("{:.0}", std::f32::consts::PI));
    }

    #[test]
    fn fixed_one() {
        let locale = Locale::new_from_locale_identifier("en-US");
        let modifier = Fixed;
        let result = modifier.transform(
            &locale,
            &&&*std::f32::consts::PI.to_string(),
            &[&(None, "1")],
        );
        assert_eq!(result, format!("{:.1}", std::f32::consts::PI));
    }

    #[test]
    fn fixed_five_dutch() {
        let locale = Locale::new_from_locale_identifier("nl-NL");
        let modifier = Fixed;
        let result = modifier.transform(
            &locale,
            &&&*std::f32::consts::PI.to_string(),
            &[&(None, "5")],
        );
        assert_eq!(
            result,
            format!("{:.5}", std::f32::consts::PI).replace(".", ",")
        );
    }

    #[test]
    fn fixed_big_dutch() {
        let locale = Locale::new_from_locale_identifier("nl-NL");
        let modifier = Fixed;
        let result = modifier.transform(&locale, &&"31415.92773", &[&(None, "5")]);
        assert_eq!(result, "31.415,92773");
    }

    #[test]
    fn fixed_big_egyptian_arabic() {
        let locale = Locale::new_from_locale_identifier("ar-EG");
        let modifier = Fixed;
        let result = modifier.transform(&locale, &&"31415.92773", &[&(None, "5")]);
        assert_eq!(result, "٣١٬٤١٥٫٩٢٧٧٣");
    }
}
