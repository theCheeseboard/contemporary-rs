use chrono::{DateTime, TimeZone};
use std::hash::{Hash, Hasher};

#[doc(hidden)]
pub trait ModifierTransformHash {
    fn hash<H: Hasher>(&self, state: &mut H);
}

macro_rules! implement_modifier_transform_hash_with_hash {
    ($t:ty) => {
        impl ModifierTransformHash for $t {
            fn hash<H: Hasher>(&self, state: &mut H) {
                Hash::hash(self, state);
            }
        }
    };
}

implement_modifier_transform_hash_with_hash!(&str);
implement_modifier_transform_hash_with_hash!(String);

impl ModifierTransformHash for f64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.to_bits());
    }
}

#[cfg(feature = "chrono")]
impl<T> ModifierTransformHash for DateTime<T>
where
    T: TimeZone,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self, state);
    }
}
