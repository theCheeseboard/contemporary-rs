use cntp_i18n_build_core::load::{self, TranslationEntry};
use rustc_hash::{FxHashMap, FxHashSet};
use std::env::args;
use std::sync::Arc;
use std::{env, iter, path::Path, sync::LazyLock};

use crate::{
    config::I18N_CONFIG,
    parse_raw_string::{I18nFullStringPart, parse_raw_string},
};

// keep TranslationEntry and load::translation as is
// then we make a new function that takes the outputted translation entries from load::translation
// and parses them into a new type
//
// then we use the output of *that* function as the data we store in the below variable
// instead of the raw output of
//

#[derive(Clone)]
pub enum ParsedTranslationEntry {
    Entry(Vec<I18nFullStringPart>),
    PluralEntry(FxHashMap<String, Vec<I18nFullStringPart>>),
}

fn is_running_under_rust_analyzer() -> bool {
    if let Some(application_path) = args().next() {
        // HACK: Is this really the best way to check if we're running under rust-analyzer?
        application_path.contains("rust-analyzer")
    } else {
        false
    }
}

fn read_translation_file() -> FxHashMap<String, FxHashMap<String, ParsedTranslationEntry>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");

    let config = &*I18N_CONFIG;
    let catalog_files = config.i18n.catalog_files(Path::new(&manifest_dir));

    catalog_files
        .iter()
        .map(|file| {
            let language = file.file_stem().unwrap().to_str().unwrap().to_string();
            let decoded_file = load::translation(file).unwrap();

            (
                language,
                decoded_file
                    .into_iter()
                    .map(|(key, entry)| match entry {
                        TranslationEntry::Entry(entry) => {
                            (key, ParsedTranslationEntry::Entry(parse_raw_string(&entry)))
                        }
                        TranslationEntry::PluralEntry(hash_map) => (
                            key,
                            ParsedTranslationEntry::PluralEntry(
                                hash_map
                                    .into_iter()
                                    .map(|(plural_group, string)| {
                                        (plural_group, parse_raw_string(&string))
                                    })
                                    .collect(),
                            ),
                        ),
                    })
                    .collect(),
            )
        })
        .collect()
}

static TRANSLATION_FILE_CACHE: LazyLock<
    Arc<FxHashMap<String, FxHashMap<String, ParsedTranslationEntry>>>,
> = LazyLock::new(|| Arc::new(read_translation_file()));

pub fn translation_file_cache() -> Arc<FxHashMap<String, FxHashMap<String, ParsedTranslationEntry>>>
{
    if is_running_under_rust_analyzer() {
        return Arc::new(read_translation_file());
    }
    TRANSLATION_FILE_CACHE.clone()
}

fn create_variable_list() -> FxHashMap<String, Vec<String>> {
    let sets: FxHashMap<String, FxHashSet<String>> = translation_file_cache().iter().fold(
        FxHashMap::<String, FxHashSet<String>>::default(),
        |mut foldit, (_language, strings)| {
            for (key, entry) in strings.iter() {
                let variables = match entry {
                    ParsedTranslationEntry::Entry(string) => string.clone(),
                    ParsedTranslationEntry::PluralEntry(hash_map) => hash_map
                        .iter()
                        .flat_map(|(_, string)| string)
                        .cloned()
                        .collect::<Vec<_>>(),
                }
                .iter()
                .filter_map(|thing| match thing {
                    I18nFullStringPart::Static(_) => None,
                    I18nFullStringPart::Variable(name) => Some(name.to_string()),
                    I18nFullStringPart::Count => Some("count".to_string()),
                })
                .collect::<FxHashSet<_>>();

                let z = foldit.entry(key.clone()).or_default(); //.union(&variables);

                for variable in variables {
                    z.insert(variable);
                }
            }

            foldit
        },
    );

    sets.into_iter()
        .map(|(key, variables)| (key.clone(), variables.iter().cloned().collect::<Vec<_>>()))
        .chain(iter::once((
            "TR_F".to_string(),
            vec!["variable".to_string()],
        )))
        .collect()
}

static VARIABLE_LIST: LazyLock<Arc<FxHashMap<String, Vec<String>>>> =
    LazyLock::new(|| Arc::new(create_variable_list()));

pub fn variable_list() -> Arc<FxHashMap<String, Vec<String>>> {
    if is_running_under_rust_analyzer() {
        return Arc::new(create_variable_list());
    }
    VARIABLE_LIST.clone()
}
