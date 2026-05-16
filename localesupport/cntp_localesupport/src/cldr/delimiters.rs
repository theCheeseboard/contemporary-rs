use crate::Locale;
use rust_embed::RustEmbed;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "cldr/cldr-misc-full/main"]
#[include = "*/delimiters.json"]
struct CldrDelimitersData;

impl CldrDelimitersData {
    pub fn delimiters_for_language(lang: &str) -> Option<Delimiters> {
        for range in Locale::split_language_range(lang) {
            if let Some(file) = Self::get(format!("{range}/delimiters.json").as_str()) {
                let mut root: DelimitersFile = serde_json::from_slice(file.data.as_ref()).unwrap();
                let mut delimiters = root.main.remove(&range).unwrap().delimiters;
                if range.starts_with("fr") && delimiters.quotation_start == "«" {
                    delimiters = Delimiters {
                        quotation_start: "«\u{A0}".to_string(),
                        quotation_end: "\u{A0}»".to_string(),
                        alternate_quotation_start: delimiters.alternate_quotation_start,
                        alternate_quotation_end: delimiters.alternate_quotation_end,
                    }
                }
                return Some(delimiters);
            }
        }
        None
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DelimitersFile {
    main: HashMap<String, DelimitersFileEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DelimitersFileEntry {
    delimiters: Delimiters,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Delimiters {
    pub quotation_start: String,
    pub quotation_end: String,
    pub alternate_quotation_start: String,
    pub alternate_quotation_end: String,
}

impl Default for Delimiters {
    fn default() -> Self {
        Delimiters {
            quotation_start: "\"".to_string(),
            quotation_end: "\"".to_string(),
            alternate_quotation_start: "'".to_string(),
            alternate_quotation_end: "'".to_string(),
        }
    }
}

impl Delimiters {
    pub fn new(language: &str) -> Self {
        CldrDelimitersData::delimiters_for_language(language).unwrap_or_default()
    }
}
