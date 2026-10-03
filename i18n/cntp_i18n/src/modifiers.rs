use cntp_localesupport::Locale;
use cntp_localesupport::modifiers::{ModifierVariable, StringModifier};
use rustc_hash::FxHasher;
use std::hash::Hash;

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
    pub(crate) &'a dyn StringModifier<String>,
    pub(crate) &'a [ModifierVariable<'a>],
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

#[doc(hidden)]
impl Variable<'_> {
    pub(crate) fn hash_value(&self, state: &mut FxHasher) {
        match self {
            Variable::Modified(modifier, _) => {
                modifier.hash(state);
            }
            Variable::String(string) => string.hash(state),
            Variable::Count(count) => count.hash(state),
        }
    }
}

/// A tuple pair representing a lookup variable
#[doc(hidden)]
pub type LookupVariable<'a> = &'a (&'a str, Variable<'a>);
