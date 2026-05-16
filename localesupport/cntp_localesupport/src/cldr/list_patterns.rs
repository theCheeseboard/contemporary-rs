use crate::Locale;
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "cldr/cldr-misc-full/main"]
#[include = "*/listPatterns.json"]
struct CldrListPatternsData;

impl CldrListPatternsData {
    pub fn list_patterns_for_language(lang: &str) -> Option<ListPatterns> {
        for range in Locale::split_language_range(lang) {
            if let Some(file) = Self::get(format!("{range}/listPatterns.json").as_str()) {
                let mut root: ListPatternsFile =
                    serde_json::from_slice(file.data.as_ref()).unwrap();
                let list_patterns = root.main.remove(&range).unwrap().list_patterns;
                return Some(list_patterns);
            }
        }
        None
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListPatternsFile {
    main: HashMap<String, ListPatternsFileEntry>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ListPatternsFileEntry {
    list_patterns: ListPatterns,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ListPatterns {
    #[serde(rename = "listPattern-type-standard")]
    pub standard: ListPattern,

    #[serde(rename = "listPattern-type-or")]
    pub or: ListPattern,

    #[serde(rename = "listPattern-type-unit")]
    pub unit: ListPattern,

    #[serde(rename = "listPattern-type-unit-narrow")]
    pub unit_narrow: ListPattern,

    #[serde(rename = "listPattern-type-unit-short")]
    pub unit_short: ListPattern,

    #[serde(rename = "listPattern-type-standard-narrow")]
    pub standard_narrow: ListPattern,

    #[serde(rename = "listPattern-type-standard-short")]
    pub standard_short: ListPattern,

    #[serde(rename = "listPattern-type-or-narrow")]
    pub or_narrow: ListPattern,

    #[serde(rename = "listPattern-type-or-short")]
    pub or_short: ListPattern,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListPattern {
    pub start: String,
    pub middle: String,
    pub end: String,

    #[serde(rename = "2")]
    pub two: String,
}

impl Default for ListPattern {
    fn default() -> Self {
        ListPattern {
            start: "{0}, {1}".into(),
            middle: "{0}, {1}".into(),
            end: "{0}, and {1}".into(),
            two: "{0} and {1}".into(),
        }
    }
}

impl ListPatterns {
    pub fn new(language: &str) -> Self {
        CldrListPatternsData::list_patterns_for_language(language).unwrap_or_default()
    }
}
