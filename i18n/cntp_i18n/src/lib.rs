//! # `cntp_i18n`
//!
//! `cntp_i18n` is an ergonomic internationalization (i18n) library for Rust.
//!
//! ## Setup
//!
//! Add `cntp_i18n` to your dependencies:
//!
//! ```toml
//! [dependencies]
//! cntp_i18n = { git = "https://github.com/vicr123/contemporary-rs" }
//! ```
//!
//! ### Loading translations
//!
//! Initialize the translation system at application startup using [`tr_load!`] to load
//! the translations into the binary. This must be done before calling [`tr!`] or [`trn!`].
//!
//! ```rust,ignore
//! use cntp_i18n::{I18N_MANAGER, tr_load};
//!
//! fn main() {
//!     I18N_MANAGER.write().unwrap().load_source(tr_load!());
//!
//!     // Now you can use tr! and trn! macros
//!     println!("{}", tr!("HELLO_WORLD", "Hello World!"));
//! }
//! ```
//!
//! ## Translation files
//! Translation files are stored in the `translations` directory (configurable via `i18n.toml`):
//!
//! - `en.json` - English translations (or your default language)
//! - `meta.json` - Metadata about each translation string
//! - `fr.json`, `de.json`, etc. - Additional language files
//!
//! Translations can be embedded in your application's macros. There are two ways to generate
//! translation files from tr! and trn! macros:
//!
//! ### `cntp_i18n_gen`
//!
//! Add the `cntp_i18n_gen` to your build dependencies:
//!
//! ```toml
//! [build-dependencies]
//! cntp_i18n_gen = { git = "https://github.com/vicr123/contemporary-rs" }
//! ```
//!
//! Then, add the code to generate the files to your build.rs script:
//!
//! ```rust,ignore
//! use std::{env, path::PathBuf};
//!
//! fn main() {
//!     let path: PathBuf = env::var("CARGO_MANIFEST_DIR")
//!         .expect("CARGO_MANIFEST_DIR is not set")
//!         .into();
//!
//!     cntp_i18n_gen::generate_default(&path);
//! }
//! ```
//!
//! ### `cargo-cntp-i18n`
//!
//! Alternatively, you can use the `cargo cntp-i18n generate` command to manually generate
//! translation files instead of using a build script. See cargo-cntp-i18n for more details.
//!
//! ## Usage
//!
//! Use `tr!` for simple, non-plural strings:
//!
//! ```rust,ignore
//! // Basic usage
//! tr!("BUTTON_SIGN_IN", "Sign In");
//!
//! // With variable substitution
//! tr!("GREETING", "Hello, {{name}}!", name = user_name);
//!
//! // With modifiers (e.g., quoting)
//! tr!("QUOTED_NAME", "Your name is {{name}}.", name:quote = user_name);
//! ```
//!
//! ### Plural strings
//!
//! Use `trn!` for strings that vary based on count:
//!
//! ```rust,ignore
//! trn!(
//!     "UNREAD_EMAILS",
//!     "You have {{count}} unread email.",
//!     "You have {{count}} unread emails.",
//!     count = email_count
//! );
//! ```
//!
//! ## Configuration
//!
//! Create an `i18n.toml` file in your project root to customize behavior:
//!
//! ```toml
//! [i18n]
//! default_language = "en"
//! translation_directory = "translations"
//! match_line_endings = true
//! ```
//!
//! ## Feature flags
//!
//! - **`gpui`** - Enables automatic conversion of translation results to GPUI's `SharedString`.
//! This allows the text to be used as an child easily, without any additional function calls.
//! - **`chrono`** - Enables `chrono` date/time type support in the [`Date`] modifier.
//!
//! ## Architecture
//!
//! This crate re-exports types from several internal crates:
//!
//! - Macros ([`tr!`], [`trn!`], [`trf!`], [`tr_noop!`], [`trn_noop!`], [`tr_load!`]) from `cntp_i18n_macros`
//! - Core types ([`I18nSource`], [`I18nEntry`], etc.) from `cntp_i18n_core`
//! - Locale support ([`Locale`], [`LocaleFormattable`], modifiers) from `cntp_localesupport`
//!
//! For build-time generation, use `cntp_i18n_gen` in your `build.rs`.
//!
//! [`Date`]: modifiers::Date

#![warn(missing_docs)]

pub use cntp_i18n_macros::{tr, tr_load, tr_noop, trf, trn, trn_noop};
use cntp_localesupport::modifiers::ModifierVariable;
use once_cell::sync::Lazy;
use quick_cache::sync::Cache;
use rustc_hash::FxHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::hardcoded_i18n_source::HardcodedI18nSource;
pub use cntp_i18n_core::{
    I18nEntry, I18nPluralStringEntry, I18nSource, I18nStringEntry, I18nStringPart,
    string::I18nString,
};
pub use cntp_localesupport::locale_formattable::LocaleFormattable;
pub use cntp_localesupport::modifiers::{Date, Quote, StringModifier};
pub use cntp_localesupport::{LayoutDirection, ListFunction, Locale};
pub use phf;

mod hardcoded_i18n_source;

#[cfg(feature = "pseudotranslation")]
mod pseudotranslation;

/// The global i18n manager instance.
///
/// This is the central point for managing translations in your application. It holds
/// all loaded translation sources and the current locale settings.
///
/// # Usage
///
/// Load translation sources at application startup:
///
/// ```rust,ignore
/// use cntp_i18n::{I18N_MANAGER, tr_load};
///
/// fn main() {
///     // Load translations (typically done once at startup)
///     I18N_MANAGER.write().unwrap().load_source(tr_load!());
///
///     // Change locale if needed
///     I18N_MANAGER.write().unwrap().locale = Locale::new_from_locale_identifier("fr");
/// }
/// ```
pub static I18N_MANAGER: Lazy<SharedI18nManager> =
    Lazy::new(|| SharedI18nManager(RwLock::new(I18nManager::default())));

static KEY_HASHES: Lazy<RwLock<HashMap<String, Vec<u64>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Manages the state of the i18n system in the application.
///
/// The `I18nManager` is responsible for:
/// - Keeping track of all loaded translation sources
/// - Storing the current locale settings
/// - Caching translated strings for performance
/// - Looking up translations from the appropriate source
///
/// # Translation Lookup Order
///
/// When looking up a translation, the manager searches through sources in reverse order based
/// on when they were loaded. The first source that provides a non-empty translation wins.
/// If no translation is found, the key itself is returned.
pub struct I18nManager {
    sources: Vec<Box<dyn I18nSource>>,
    /// The current locale used for translation lookups.
    ///
    /// This determines which language translations are retrieved in, as well as
    /// locale-specific formatting (numbers, dates, etc.).
    pub locale: Locale,
    cache: Cache<u64, I18nString>,

    cache_eviction_callbacks: Vec<Arc<dyn Fn() + Send + Sync>>,
}

/// Internal trait for type-erased string modifier transformations.
///
/// This trait is used internally by the macro system to handle modifiers
/// without knowing the concrete input type at compile time.
#[doc(hidden)]
pub trait ErasedStringModifierTransform {
    /// Apply the transformation using the given locale.
    fn transform(&self, locale: &Locale) -> String;
    /// Hash the input value for cache key generation.
    fn hash(&self, state: &mut FxHasher);
}

/// Internal type for the first modifier in a chain.
///
/// This is used by the macro expansion and should not be used directly.
#[doc(hidden)]
pub struct BaseStringModifierInvocation<'a, T: ?Sized + Hash>(
    &'a dyn StringModifier<&'a T>,
    &'a [ModifierVariable<'a>],
    &'a T,
);

impl<'a, T: ?Sized + Hash> BaseStringModifierInvocation<'a, T> {
    /// Create a new base modifier invocation.
    #[doc(hidden)]
    pub fn new(
        modifier: &'a dyn StringModifier<&'a T>,
        variables: &'a [ModifierVariable<'a>],
        input: &'a T,
    ) -> Self {
        BaseStringModifierInvocation(modifier, variables, input)
    }
}

impl<'a, T: ?Sized + Hash> ErasedStringModifierTransform for BaseStringModifierInvocation<'a, T> {
    fn transform(&self, locale: &Locale) -> String {
        let BaseStringModifierInvocation(modifier, variables, input) = self;
        modifier.transform(locale, input, variables)
    }

    fn hash(&self, state: &mut FxHasher) {
        let BaseStringModifierInvocation(_, _, input) = self;
        input.hash(state);
    }
}

/// Internal type for subsequent modifiers in a chain.
///
/// This is used by the macro expansion and should not be used directly.
#[doc(hidden)]
pub struct SubsequentStringModifierInvocation<'a>(
    &'a dyn StringModifier<String>,
    &'a [ModifierVariable<'a>],
);

impl<'a> SubsequentStringModifierInvocation<'a> {
    /// Create a new subsequent modifier invocation.
    #[doc(hidden)]
    pub fn new(
        modifier: &'a dyn StringModifier<String>,
        variables: &'a [ModifierVariable<'a>],
    ) -> Self {
        SubsequentStringModifierInvocation(modifier, variables)
    }
}

/// Internal representation of a variable passed to translation lookups.
///
/// This is used by the macro expansion and should not be used directly.
#[doc(hidden)]
pub enum Variable<'a> {
    /// A variable with one or more modifiers applied.
    Modified(
        &'a dyn ErasedStringModifierTransform,
        &'a [SubsequentStringModifierInvocation<'a>],
    ),
    /// A plain string variable.
    String(String),
    /// A count variable for plural lookups.
    Count(isize),
}

impl Variable<'_> {
    fn hash_value(&self, state: &mut FxHasher) {
        match self {
            Variable::Modified(modifier, _) => {
                modifier.hash(state);
            }
            Variable::String(string) => string.hash(state),
            Variable::Count(count) => count.hash(state),
        }
    }
}

type LookupVariable<'a> = &'a (&'a str, Variable<'a>);

impl I18nManager {
    /// Subscribes a callback to be invoked during cache eviction events.
    ///
    /// This method allows users to register a callback function that will be executed
    /// whenever a cache eviction occurs. This can be used to rerender the UI when
    /// translations change, for example.
    ///
    /// # Parameters
    /// - `callback`: A closure or function that implements `Fn() + Send + Sync + 'static`.
    ///   This callback will be called when a cache eviction event is triggered.
    pub fn subscribe_to_cache_eviction(&mut self, callback: impl Fn() + Send + Sync + 'static) {
        self.cache_eviction_callbacks.push(Arc::new(callback));
    }

    /// Look up a translation with caching.
    ///
    /// This method first checks the cache for a previously resolved translation.
    /// If not found, it performs a full lookup and caches the result.
    ///
    /// # Note
    ///
    /// This is an internal method used by the [`tr!`] and [`trn!`] macros.
    /// You should use those macros instead of calling this directly.
    ///
    /// If you need functionality not provided by the macros, please file an issue.
    #[doc(hidden)]
    pub fn lookup_cached<'a>(
        &self,
        key: &str,
        variables: &'a [Option<LookupVariable<'a>>],
        lookup_crate: &str,
        hash: u64,
        locale_override: Option<&Locale>,
    ) -> I18nString {
        let mut state = FxHasher::default();
        hash.hash(&mut state);
        for variable in variables.iter().flatten() {
            variable.1.hash_value(&mut state);
        }
        if let Some(locale) = locale_override {
            (&locale).hash(&mut state);
        }
        let full_call_hash = state.finish();

        self.cache.get(&full_call_hash).clone().unwrap_or_else(|| {
            let result = self.lookup(key, variables, lookup_crate, locale_override);
            self.cache.insert(full_call_hash, result.clone());
            KEY_HASHES
                .write()
                .unwrap()
                .entry(key.to_string())
                .or_default()
                .push(full_call_hash);
            result
        })
    }

    /// Look up a translation from the loaded sources.
    ///
    /// This method searches through all loaded translation sources to find a match
    /// for the given key. Variable substitution and plural resolution are handled
    /// automatically.
    ///
    /// # Note
    ///
    /// This is an internal method used by the [`tr!`] and [`trn!`] macros.
    /// You should use those macros instead of calling this directly.
    ///
    /// If you need functionality not provided by the macros, please file an issue.
    #[doc(hidden)]
    pub fn lookup<'a>(
        &self,
        key: &str,
        variables: &'a [Option<LookupVariable<'a>>],
        lookup_crate: &str,
        locale_override: Option<&Locale>,
    ) -> I18nString {
        let locale = locale_override.unwrap_or(&self.locale);

        for source in self.sources.iter().rev() {
            let Some(entry) = source.lookup(locale, key, lookup_crate) else {
                continue;
            };

            let parts: &[I18nStringPart] = match &entry {
                I18nEntry::Entry(entry) => entry,
                I18nEntry::PluralEntry(entry) => {
                    let (_, count) = variables
                        .iter()
                        .find(|variable| match variable {
                            Some((name, _)) => *name == "count",
                            None => false,
                        })
                        .and_then(Option::as_deref)
                        .unwrap_or_else(|| {
                            panic!(
                                "Resolved plural string for {key}, but no count variable \
                                provided for substitution",
                            )
                        });

                    match count {
                        Variable::Count(count) => {
                            // Plural entries with count substitution always need allocation
                            let plural_parts = entry.lookup(*count, locale);
                            return self.resolve_parts_to_string(&plural_parts, variables, locale);
                        }
                        Variable::String(string) => {
                            panic!("Count variable ({string}) not of type isize")
                        }
                        Variable::Modified(_inital, _subsequent) => {
                            panic!("Cannot modify count variable")
                        }
                    }
                }
            };

            // Fast path: if there's exactly one static part with no variables, return borrowed
            if parts.len() == 1
                && let I18nStringPart::Static(s) = &parts[0]
                && !s.is_empty()
            {
                #[cfg(feature = "pseudotranslation")]
                {
                    if std::env::var("CNTP_I18N_ENABLE_PSEUDOTRANSLATION").is_ok() {
                        return pseudotranslation::contain(
                            &pseudotranslation::mangle(&s.to_string()),
                            s.len(),
                        )
                        .into();
                    }
                }

                return s.clone();
            }

            // Check if all parts are static (no variable substitution needed)
            let all_static = parts.iter().all(|p| matches!(p, I18nStringPart::Static(_)));

            if all_static {
                if parts.is_empty() {
                    continue;
                }
                let resolved: String = parts
                    .iter()
                    .filter_map(|p| {
                        if let I18nStringPart::Static(s) = p {
                            Some(s.as_ref())
                        } else {
                            None
                        }
                    })
                    .collect();

                if resolved.is_empty() {
                    continue;
                }

                return resolved.into();
            }

            // Slow path: need variable substitution
            let resolved = self.resolve_parts_to_string(parts, variables, locale);

            // If the translation is empty, fall back to the next source
            if resolved.is_empty() {
                continue;
            }

            return resolved;
        }

        // None of the translation sources we have were able to find a key so just return the key
        key.to_string().into()
    }

    fn resolve_parts_to_string<'a>(
        &self,
        parts: &[I18nStringPart],
        variables: &'a [Option<LookupVariable<'a>>],
        locale: &Locale,
    ) -> I18nString {
        let mut result = String::new();

        #[cfg(feature = "pseudotranslation")]
        let mut len = 0;

        for part in parts {
            match part {
                I18nStringPart::Static(borrowed) => {
                    #[cfg(feature = "pseudotranslation")]
                    {
                        if std::env::var("CNTP_I18N_ENABLE_PSEUDOTRANSLATION").is_ok() {
                            len += borrowed.len();
                            result.push_str(
                                pseudotranslation::mangle(&borrowed.to_string()).as_ref(),
                            );
                        } else {
                            result.push_str(borrowed)
                        }
                    }

                    #[cfg(not(feature = "pseudotranslation"))]
                    result.push_str(borrowed)
                }
                I18nStringPart::Variable(variable, idx) => {
                    let substituted_variable = variables
                        .get(*idx)
                        .and_then(Option::as_deref)
                        .filter(|(name, _)| *name == variable.as_ref())
                        .or_else(|| {
                            // Fallback for if idx is out of bounds or doesn't match the variable name
                            variables
                                .iter()
                                .filter_map(Option::as_deref)
                                .find(|(name, _)| *name == variable.as_ref())
                        })
                        .map(|(_, variable)| variable);

                    match substituted_variable {
                        Some(Variable::Modified(initial, subsequent)) => {
                            let modify_result = subsequent
                                .iter()
                                .fold(initial.transform(locale), |v, modi| {
                                    modi.0.transform(locale, v, modi.1)
                                });

                            #[cfg(feature = "pseudotranslation")]
                            {
                                if std::env::var("CNTP_I18N_ENABLE_PSEUDOTRANSLATION").is_ok() {
                                    result.push_str(&pseudotranslation::contain_variable(
                                        &modify_result,
                                    ));
                                } else {
                                    result.push_str(&modify_result);
                                }
                            }

                            #[cfg(not(feature = "pseudotranslation"))]
                            result.push_str(&modify_result);
                        }
                        Some(Variable::String(str)) => {
                            #[cfg(feature = "pseudotranslation")]
                            {
                                if std::env::var("CNTP_I18N_ENABLE_PSEUDOTRANSLATION").is_ok() {
                                    result.push_str(&pseudotranslation::contain_variable(str))
                                } else {
                                    result.push_str(str)
                                }
                            }

                            #[cfg(not(feature = "pseudotranslation"))]
                            result.push_str(str)
                        }
                        Some(Variable::Count(_)) => {
                            panic!("Unexpected count variable")
                        }
                        None => {
                            result.push_str("{{");
                            result.push_str(variable);
                            result.push_str("}}");
                        }
                    }
                }
                I18nStringPart::Count(_) => result.push_str("{{count}}"),
            }
        }

        #[cfg(feature = "pseudotranslation")]
        if std::env::var("CNTP_I18N_ENABLE_PSEUDOTRANSLATION").is_ok() {
            return I18nString::Owned(pseudotranslation::contain(&result, len).into());
        }

        I18nString::Owned(result.into())
    }
}

impl Default for I18nManager {
    fn default() -> Self {
        I18nManager {
            sources: vec![Box::new(HardcodedI18nSource)],
            locale: Locale::current(),
            cache: Cache::new(500),
            cache_eviction_callbacks: vec![],
        }
    }
}

/// The shared I18nManager, used in [I18N_MANAGER].
pub struct SharedI18nManager(RwLock<I18nManager>);

impl SharedI18nManager {
    /// Clear the translation cache.
    ///
    /// This can be used in the event that a dynamic translation provider is used, and the cache
    /// becomes stale.
    pub fn clear_cache(&self) {
        self.0.write().unwrap().cache.clear();
        KEY_HASHES.write().unwrap().clear();
    }

    /// Load a translation source into the manager.
    ///
    /// Translation sources are searched in reverse order of when they are loaded. This allows
    /// you to override translations by loading a more specific source after a
    /// general one.
    ///
    /// Loading a new translation source will clear the translation cache.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use cntp_i18n::{I18N_MANAGER, tr_load};
    ///
    /// // Load the main application translations
    /// I18N_MANAGER.load_source(tr_load!());
    ///
    /// // Optionally load additional/override translations
    /// // I18N_MANAGER.load_source(custom_source);
    /// ```
    ///
    /// # Typical Usage
    ///
    /// Most applications will call this once at startup with `tr_load!()`:
    ///
    /// ```rust,ignore
    /// fn main() {
    ///     I18N_MANAGER.load_source(tr_load!());
    ///     // ... rest of application
    /// }
    /// ```
    pub fn load_source(&self, source: impl I18nSource + 'static) {
        {
            let mut write = self.write().unwrap();
            write.sources.push(Box::new(source));

            // Clear the cache because new translations might be available
            write.cache.clear();
        }

        KEY_HASHES.write().unwrap().clear();
    }

    /// Evicts all translations for a specified key from the cache
    ///
    /// This is intended to be used by translation providers to notify the
    /// translation system when translations for a key have changed or become invalid.
    pub fn evict_key(&self, key: &str) {
        if let Some(hashes) = KEY_HASHES.read().unwrap().get(key) {
            for hash in hashes {
                self.write().unwrap().cache.remove(hash);
            }
        }

        for callback in self.read().unwrap().cache_eviction_callbacks.clone() {
            callback();
        }
    }

    /// Retrieves the current locale
    pub fn locale(&self) -> Locale {
        self.read().unwrap().locale.clone()
    }
}

impl Deref for SharedI18nManager {
    type Target = RwLock<I18nManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
