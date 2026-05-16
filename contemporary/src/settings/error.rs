use std::io::Error;

pub enum SettingsError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}

impl From<std::io::Error> for SettingsError {
    fn from(value: Error) -> Self {
        SettingsError::IoError(value)
    }
}

impl From<serde_json::Error> for SettingsError {
    fn from(value: serde_json::Error) -> Self {
        SettingsError::SerdeError(value)
    }
}
