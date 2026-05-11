pub mod config;

#[cfg(feature = "blueprint_detection")]
mod blueprint_auto_detection;

use crate::config::{
    ContemporaryConfigApplicationDef, ContemporaryConfigConfigDef, ContemporaryConfigDeployment,
    ContemporaryConfigDeploymentDef,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use tracing::warn;

#[derive(Deserialize)]
pub struct ContemporaryConfig {
    pub config: Option<ContemporaryConfigConfigDef>,
    pub application: ContemporaryConfigApplicationDef,
    pub deployment: ContemporaryConfigDeploymentDef,

    #[serde(skip)]
    translations: HashMap<String, HashMap<String, String>>,

    #[serde(skip)]
    path: PathBuf,
}

impl ContemporaryConfig {
    pub fn new_from_path(path: PathBuf) -> Option<Self> {
        if path.exists() {
            let Ok(mut file) = OpenOptions::new().read(true).open(&path) else {
                return None;
            };
            let mut contents = String::new();
            let Ok(_) = file.read_to_string(&mut contents) else {
                return None;
            };

            let Ok(mut config) = toml::from_str::<ContemporaryConfig>(&contents) else {
                return None;
            };

            config.load_translations(path.parent().unwrap().into());
            config.path = path;

            Some(config)
        } else {
            None
        }
    }

    pub fn new_from_build_env() -> Option<Self> {
        let Ok(cargo_manifest_dir) = env::var("CARGO_MANIFEST_DIR") else {
            return None;
        };

        let manifest_dir = PathBuf::from(cargo_manifest_dir);
        let contemporary_path = manifest_dir.join("Contemporary.toml");
        Self::new_from_path(contemporary_path)
    }

    fn load_translations(&mut self, base_path: PathBuf) {
        let Some(ref config_config) = self.config else {
            return;
        };

        let Some(ref i18n_dir) = config_config.i18n_dir else {
            return;
        };

        let translations_path = base_path.join(i18n_dir);
        let Ok(translation_files) = translations_path.read_dir() else {
            warn!("Unable to read translations directory");
            return;
        };

        let translation_files = translation_files.flatten();

        for file in translation_files {
            let Ok(meta) = file.metadata() else {
                continue;
            };

            if !meta.is_file() {
                continue;
            }

            let path = file.path();
            if !path.extension().unwrap_or_default().eq("json") {
                continue;
            }

            let lang = path
                .with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let Ok(mut translation_file) = OpenOptions::new().read(true).open(&path) else {
                warn!(
                    "Unable to open translation file: {}",
                    path.to_str().unwrap()
                );
                continue;
            };

            let mut json_contents = String::new();
            let Ok(_) = translation_file.read_to_string(&mut json_contents) else {
                warn!(
                    "Unable to read translation file: {}",
                    path.to_str().unwrap()
                );
                continue;
            };

            let Ok(translations) = serde_json::from_str(&json_contents) else {
                warn!(
                    "Unable to parse translation file: {}",
                    path.to_str().unwrap()
                );
                continue;
            };

            self.translations.insert(lang, translations);
        }
    }

    pub fn deployment(&self, arch: &str) -> ContemporaryConfigDeployment {
        let deployment = self.deployment.clone();
        let specific_deployment = if let Some(x) = self.deployment.clone().children.get(arch) {
            x.clone()
        } else {
            ContemporaryConfigDeploymentDef::default()
        };

        let mut extra_info_plist_attributes = HashMap::new();
        if let Some(extra_attributes) = deployment.extra_info_plist_attributes {
            extra_info_plist_attributes.extend(extra_attributes);
        }
        if let Some(extra_attributes) = specific_deployment.extra_info_plist_attributes {
            extra_info_plist_attributes.extend(extra_attributes);
        }

        ContemporaryConfigDeployment {
            application_name: self.resolve_localised_string(
                specific_deployment
                    .application_name
                    .or(deployment.application_name),
            ),
            application_generic_name: self.resolve_localised_string(
                specific_deployment
                    .application_generic_name
                    .or(deployment.application_generic_name),
            ),
            application_machine_name: specific_deployment
                .application_machine_name
                .or(deployment.application_machine_name),
            organization_name: specific_deployment
                .organization_name
                .or(deployment.organization_name),
            desktop_entry: specific_deployment
                .desktop_entry
                .or(deployment.desktop_entry),

            icon: specific_deployment.icon.or(deployment.icon),
            contemporary_base_icon: specific_deployment
                .contemporary_base_icon
                .or(deployment.contemporary_base_icon),

            handled_url_schemes: specific_deployment
                .handled_url_schemes
                .or(deployment.handled_url_schemes),

            // Linux only
            desktop_entry_categories: specific_deployment
                .desktop_entry_categories
                .or(deployment.desktop_entry_categories),

            // macOS only
            apple_localisation_dir: self.resolve_localised_string(
                specific_deployment
                    .apple_localisation_dir
                    .or(deployment.apple_localisation_dir),
            ),
            extra_info_plist_attributes: extra_info_plist_attributes
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        self.resolve_localised_string(Some(value.clone())).unwrap(),
                    )
                })
                .collect(),
            minimum_system_version: specific_deployment
                .minimum_system_version
                .or(deployment.minimum_system_version)
                .unwrap_or("10.15".to_string()),
            disk_image_background: specific_deployment
                .disk_image_background
                .or(deployment.disk_image_background),
            supports_automatic_graphics_switching: specific_deployment
                .supports_automatic_graphics_switching
                .or(deployment.supports_automatic_graphics_switching)
                .unwrap_or(true),

            is_blueprint: self.is_blueprint(),
        }
    }

    fn resolve_localised_string(&self, string: Option<String>) -> Option<LocalisedString> {
        let string = string?;

        if let Some(key) = string.strip_prefix("t:") {
            let mut string_translations = HashMap::new();
            for (lang, translations) in &self.translations {
                let Some(translation) = translations.get(key) else {
                    continue;
                };

                string_translations.insert(lang.clone(), translation.clone());
            }
            if string_translations.is_empty() {
                string_translations.insert("en".to_string(), string.clone());
            }
            Some(LocalisedString::Localised(string_translations))
        } else {
            Some(LocalisedString::Hardcoded(string))
        }
    }

    pub fn available_localisations(&self) -> Vec<String> {
        self.translations.keys().cloned().collect()
    }

    pub fn is_blueprint(&self) -> bool {
        let blueprint_configuration = self
            .config
            .clone()
            .and_then(|config| config.blueprint)
            .unwrap_or("auto".into());

        match blueprint_configuration.as_str() {
            "auto" => cfg_select! {
                feature = "blueprint_detection" => {
                    blueprint_auto_detection::autodetect_blueprint(&self.path).unwrap_or(false)
                },
                _ => panic!(
                    "Blueprint auto configuration selected but cntp_config not built \
                    with blueprint_detection feature"
                )
            },
            "false" => false,
            "true" => true,
            _ => panic!("Unknown blueprint configuration: {blueprint_configuration}"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LocalisedString {
    Hardcoded(String),
    Localised(HashMap<String, String>),
}

impl LocalisedString {
    pub fn default_value(&self) -> String {
        match self {
            LocalisedString::Hardcoded(string) => string.clone(),
            LocalisedString::Localised(map) => {
                let Some(string) = map.get("en") else {
                    return "".to_string();
                };
                string.clone()
            }
        }
    }

    pub fn resolve_language(&self, lang: &str) -> Option<String> {
        match self {
            LocalisedString::Hardcoded(string) => Some(string.clone()),
            LocalisedString::Localised(map) => map.get(lang).cloned(),
        }
    }

    pub fn resolve_languages(&self, languages: &[String]) -> Option<String> {
        if let LocalisedString::Hardcoded(hardcoded_string) = self {
            return Some(hardcoded_string.clone());
        };

        for lang in languages {
            if let Some(string) = self.resolve_language(lang) {
                return Some(string);
            }
        }
        None
    }

    pub fn resolve_languages_or_default(&self, languages: &[String]) -> String {
        self.resolve_languages(languages)
            .unwrap_or_else(|| self.default_value())
    }
}
