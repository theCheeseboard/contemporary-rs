use crate::LocalisedString;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ContemporaryConfigApplicationDef {
    pub theme_colors: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ContemporaryConfigConfigDef {
    pub blueprint: Option<String>,
    pub i18n_dir: Option<String>,
}

#[derive(Deserialize, Clone, Default)]
pub struct ContemporaryConfigDeploymentDef {
    pub application_name: Option<String>,
    pub application_generic_name: Option<String>,
    pub application_machine_name: Option<String>,
    pub organization_name: Option<String>,
    pub desktop_entry: Option<String>,
    pub icon: Option<String>,
    pub contemporary_base_icon: Option<String>,
    pub handled_url_schemes: Option<Vec<String>>,

    // Linux only
    pub desktop_entry_categories: Option<Vec<String>>,
    pub appstream_metainfo_file: Option<String>,

    // macOS only
    pub apple_localisation_dir: Option<String>,
    pub extra_info_plist_attributes: Option<HashMap<String, String>>,
    pub minimum_system_version: Option<String>,
    pub supports_automatic_graphics_switching: Option<bool>,
    pub disk_image_background: Option<String>,

    #[serde(flatten)]
    pub children: HashMap<String, ContemporaryConfigDeploymentDef>,
}

pub struct ContemporaryConfigDeployment {
    pub(crate) application_name: Option<LocalisedString>,
    pub application_generic_name: Option<LocalisedString>,
    pub application_machine_name: Option<String>,
    pub organization_name: Option<String>,
    pub desktop_entry: Option<String>,
    pub icon: Option<String>,
    pub contemporary_base_icon: Option<String>,
    pub handled_url_schemes: Option<Vec<String>>,

    // Linux only
    pub desktop_entry_categories: Option<Vec<String>>,
    pub appstream_metainfo_file: Option<String>,

    // macOS only
    pub apple_localisation_dir: Option<LocalisedString>,
    pub extra_info_plist_attributes: HashMap<String, LocalisedString>,
    pub minimum_system_version: String,
    pub disk_image_background: Option<String>,
    pub supports_automatic_graphics_switching: bool,

    pub(crate) is_blueprint: bool,
}

impl ContemporaryConfigDeployment {
    pub fn application_name(&self) -> Option<LocalisedString> {
        if !self.is_blueprint {
            return self.application_name.clone();
        };

        let Some(application_name) = &self.application_name else {
            return None;
        };

        match application_name {
            LocalisedString::Hardcoded(hardcoded_string) => Some(LocalisedString::Hardcoded(
                format!("{hardcoded_string} Blueprint"),
            )),
            LocalisedString::Localised(localisations) => Some(LocalisedString::Localised(
                localisations
                    .iter()
                    .map(|(lang, value)| (lang.to_string(), format!("{value} Blueprint")))
                    .collect(),
            )),
        }
    }
}
