use std::{
    error::Error,
};
use serde::de::{DeserializeOwned};
use crate::{auto_updated::AutoUpdated, watched_file::*};
use crate::file_format::FileFormat;

pub enum UpdateType{
    Manual,
    Automatic,
}

pub struct Watcher{
    update_type: UpdateType,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher{
            update_type: UpdateType::Automatic,
        }
    }

    pub fn update_type(mut self ,update_type: UpdateType) -> Self {
        self.update_type = update_type;
        self
    }

    pub fn watch<T: ToString>(
        &self,
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        WatchedFile::new(file_path)
    }

    pub fn watched_file_from<T: ToString>(
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        Self::new().watch(file_path)
    }

    pub fn auto_updated_from<T: ToString, W: DeserializeOwned + Send + 'static>(
        file_path: T, file_format: FileFormat
    ) -> Result<AutoUpdated<W>, Box<dyn std::error::Error>> {
        let watched_file: WatchedFile = Self::watched_file_from(file_path)?.format(file_format);
        let auto_updated_value: Result<AutoUpdated<W>, Box<dyn Error>> = watched_file.auto_updated();
        auto_updated_value
    }
}
impl Default for Watcher {
    fn default() -> Self {
        Self {
            update_type: UpdateType::Automatic,
        }
    }
}