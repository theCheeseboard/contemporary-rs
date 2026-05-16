use crate::cldr::ListPatterns;
use crate::{ListFunction, ListWidth};
use std::fmt::Display;

/// A builder struct for formatting a list of strings in a locale.
///
/// # Usage
/// The `ListBuilder` can be obtained using [`Locale::build_list`].
///
/// # Example
///
/// ```rust
/// use cntp_localesupport::{ListFunction, Locale};
///
/// let list = vec![
///     "Stacey".to_string(),
///     "Kevin".to_string(),
///     "Thomas".to_string(),
/// ];
///
/// let english = Locale::new_from_locale_identifier("en-US");
/// let german = Locale::new_from_locale_identifier("de-DE");
///
/// println!("{}", english.build_list(&list).build());
/// // Stacey, Kevin, and Thomas
/// println!("{}", english.build_list(&list).with_list_function(ListFunction::Or).build());
/// // Stacey, Kevin, or Thomas
/// println!("{}", german.build_list(&list).build());
/// // Stacey, Kevin und Thomas
/// ```
#[derive(Debug)]
pub struct ListBuilder<'patterns, 'strings> {
    patterns: &'patterns ListPatterns,
    parts: Vec<&'strings String>,

    list_function: ListFunction,
    list_width: ListWidth,
}

impl<'patterns, 'strings> ListBuilder<'patterns, 'strings> {
    pub(crate) fn new(parts: Vec<&'strings String>, patterns: &'patterns ListPatterns) -> Self {
        ListBuilder {
            patterns,
            parts,
            list_function: Default::default(),
            list_width: Default::default(),
        }
    }

    /// Set the list function to use when building the string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cntp_localesupport::{ListFunction, Locale};
    ///
    /// let list = vec![
    ///     "Stacey".to_string(),
    ///     "Kevin".to_string(),
    ///     "Thomas".to_string(),
    /// ];
    ///
    /// let english = Locale::new_from_locale_identifier("en-US");
    ///
    /// println!("{}", english.build_list(&list));
    /// // Stacey, Kevin, and Thomas
    /// println!("{}", english.build_list(&list).with_list_function(ListFunction::Or));
    /// // Stacey, Kevin, or Thomas
    /// println!("{}", english.build_list(&list).with_list_function(ListFunction::Unit));
    /// // Stacey, Kevin, Thomas
    /// ```
    pub fn with_list_function(mut self, list_function: ListFunction) -> Self {
        self.list_function = list_function;
        self
    }

    /// Set the list width to use when building the string. Mostly useful when the [`ListFunction`]
    /// is [`ListFunction::Unit`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use cntp_localesupport::{ListFunction, ListWidth, Locale};
    ///
    /// let english = Locale::new_from_locale_identifier("en-US");
    ///
    /// let list = vec![
    ///     "1 hour".to_string(),
    ///     "45 minutes".to_string(),
    ///     "30 seconds".to_string(),
    /// ];
    /// println!("{}", english.build_list(&list).with_list_function(ListFunction::Unit));
    /// // 1 hour, 45 minutes, 30 seconds
    ///
    /// let short_list = vec![
    ///     "1hr".to_string(),
    ///     "45min".to_string(),
    ///     "30sec".to_string(),
    /// ];
    /// println!("{}", english.build_list(&short_list).with_list_function(ListFunction::Unit).with_list_width(ListWidth::Short));
    /// // 1hr, 45min, 30sec
    ///
    /// let narrow_list = vec![
    ///     "1h".to_string(),
    ///     "45m".to_string(),
    ///     "30s".to_string(),
    /// ];
    /// println!("{}", english.build_list(&narrow_list).with_list_function(ListFunction::Unit).with_list_width(ListWidth::Narrow));
    /// // 1h 45m 30s
    /// ```
    pub fn with_list_width(mut self, list_width: ListWidth) -> Self {
        self.list_width = list_width;
        self
    }

    /// Build the list pattern using the configured list function and list width
    pub fn build(&self) -> String {
        let pattern = match self.list_function {
            ListFunction::Standard => match self.list_width {
                ListWidth::Wide => &self.patterns.standard,
                ListWidth::Short => &self.patterns.standard_short,
                ListWidth::Narrow => &self.patterns.standard_narrow,
            },
            ListFunction::Or => match self.list_width {
                ListWidth::Wide => &self.patterns.or,
                ListWidth::Short => &self.patterns.or_short,
                ListWidth::Narrow => &self.patterns.or_narrow,
            },
            ListFunction::Unit => match self.list_width {
                ListWidth::Wide => &self.patterns.unit,
                ListWidth::Short => &self.patterns.unit_short,
                ListWidth::Narrow => &self.patterns.unit_narrow,
            },
        };

        match self.parts.len() {
            0 => String::new(),
            1 => self.parts[0].clone(),
            2 => pattern
                .two
                .replace("{0}", self.parts[0])
                .replace("{1}", self.parts[1]),
            _ => {
                let len = self.parts.len();

                self.parts
                    .iter()
                    .enumerate()
                    .fold(String::new(), |acc: String, (i, part)| {
                        if i == 0 {
                            pattern.start.replace("{0}", part)
                        } else if i == len - 2 {
                            acc.replace("{1}", &pattern.end.replace("{0}", part))
                        } else if i == len - 1 {
                            acc.replace("{1}", part)
                        } else {
                            acc.replace("{1}", &pattern.middle.replace("{0}", part))
                        }
                    })
            }
        }
    }
}

impl From<ListBuilder<'_, '_>> for String {
    fn from(builder: ListBuilder) -> Self {
        builder.build()
    }
}

impl Display for ListBuilder<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build())
    }
}
