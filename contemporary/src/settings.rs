pub mod error;

use crate::application::Details;
use crate::settings::error::SettingsError;
use gpui::{App, Global, Path};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use tracing::error;

pub trait SettingsManager {
    fn read_settings<TData>(&self) -> Result<SettingsRwLock<TData>, SettingsError>
    where
        TData: Setting;

    fn write_settings<TData>(&self) -> Result<TData, SettingsError>
    where
        TData: Setting;

    fn erase_settings<TData>(&self) -> Result<(), SettingsError>
    where
        TData: Setting;
}

impl SettingsManager for App {
    fn read_settings<TData>(&self) -> Result<SettingsRwLock<TData>, SettingsError>
    where
        TData: Setting,
    {
        let file_path = file_path_for_data(self);
        let file = File::open(&file_path)?;
        Ok(serde_json::from_reader(file)?)
    }

    fn write_settings<TData>(&self) -> Result<TData, SettingsError>
    where
        TData: Setting,
    {
        let file_path = file_path_for_data(self);
        let data = self.read_settings()?;

        Ok(SettingsRwLock {
            data,
            path: file_path,
        })
    }

    fn erase_settings<TData>(&self) -> Result<(), SettingsError>
    where
        TData: Setting,
    {
        let file_path = file_path_for_data(self);
        std::fs::remove_file(&file_path)?;
        Ok(())
    }
}

fn file_path_for_data<TData>(cx: &App) -> PathBuf
where
    TData: Setting,
{
    let directory = TData::directory().unwrap_or_else(|| {
        let details = cx.global::<Details>();
        details.standard_dirs().unwrap().config_dir().to_path_buf()
    });

    directory.join(format!("{}.json", TData::file_name()))
}

pub struct SettingsRwLock<TData>
where
    TData: Setting,
{
    data: TData,
    path: PathBuf,
}

impl<TData> Deref for SettingsRwLock<TData>
where
    TData: Setting,
{
    type Target = TData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<TData> DerefMut for SettingsRwLock<TData>
where
    TData: Setting,
{
    fn deref_mut(&mut self) -> &mut TData {
        &mut self.data
    }
}

impl<TData> Drop for SettingsRwLock<TData>
where
    TData: Setting,
{
    fn drop(&mut self) {
        // Save the settings back into the file
        match File::create(&self.path) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.data) {
                    error!(
                        "{}: failed to serialize settings: {}",
                        TData::file_name(),
                        e
                    );
                }
            }
            Err(e) => {
                error!("{}: failed to write settings: {}", TData::file_name(), e);
            }
        }
    }
}

pub trait Setting: Serialize + for<'a> Deserialize<'a> + Default {
    fn file_name() -> &'static str;

    fn directory() -> Option<PathBuf> {
        None
    }
}
