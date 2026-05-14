use cntp_config::{ContemporaryConfig, LocalisedString};
use std::fmt::Error;
use std::path::Path;
use std::process::exit;
use tracing::error;

pub fn generate_desktop_entry(
    target_triple: &str,
    executable_path: &Path,
    contemporary_config: &ContemporaryConfig,
) -> Result<String, Error> {
    let deployment = contemporary_config.deployment(target_triple);

    let Some(application_name) = deployment.application_name() else {
        error!("No application name specified in config");
        exit(1);
    };

    let Some(desktop_entry) = deployment.desktop_entry else {
        error!("No desktop entry specified in config");
        exit(1);
    };

    let Some(desktop_entry_categories) = deployment.desktop_entry_categories else {
        error!("No desktop entry categories specified in config");
        exit(1);
    };

    let mut entry = DesktopEntry::new();
    entry.push_line_invariant("Type", "Application")?;
    entry.push_line_invariant("Version", "1.0")?;
    entry.push_line_invariant(
        "Exec",
        executable_path.file_name().unwrap().to_str().unwrap(),
    )?;
    entry.push_line_invariant("Icon", &desktop_entry)?;
    entry.push_line("Name", &application_name)?;

    if let Some(generic_name) = deployment.application_generic_name {
        entry.push_line("GenericName", &generic_name)?;
    }

    entry.push_line_invariant("Categories", &(desktop_entry_categories.join(";") + ";"))?;

    let mime_types = deployment
        .handled_url_schemes
        .unwrap_or_default()
        .iter()
        .map(|scheme| format!("x-scheme-handler/{}", scheme))
        .collect::<Vec<_>>();
    if !mime_types.is_empty() {
        entry.push_line_invariant("MimeType", &(mime_types.join(";") + ";"))?;
    }

    Ok(entry.contents)
}

struct DesktopEntry {
    pub contents: String,
}

impl DesktopEntry {
    fn new() -> Self {
        Self {
            contents: "#!/usr/bin/env xdg-open\n[Desktop Entry]\n".to_string(),
        }
    }

    fn push_line(&mut self, key: &str, value: &LocalisedString) -> Result<(), Error> {
        use std::fmt::Write;

        match value {
            LocalisedString::Hardcoded(value) => {
                writeln!(&mut self.contents, "{key}={value}")?;
            }
            LocalisedString::Localised(languages) => {
                self.push_line_invariant(key, &value.default_value())?;
                for (language, value) in languages {
                    let language = language.replace("-", "_");
                    writeln!(&mut self.contents, "{key}[{language}]={value}")?;
                }
            }
        }
        Ok(())
    }

    fn push_line_invariant(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.push_line(key, &LocalisedString::Hardcoded(value.into()))
    }
}
