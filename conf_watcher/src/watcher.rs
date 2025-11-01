use std::{
    error::Error,
};
use serde::de::{DeserializeOwned};
use crate::{auto_updated::AutoUpdated, watched_file::*};
use crate::file_format::FileFormat;

pub struct Watcher;

impl Watcher {
    pub fn new() -> Self {
        Watcher
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
        let wf = Self::watched_file_from(file_path)?.format(file_format);
        let au: Result<AutoUpdated<W>, Box<dyn Error>> = wf.auto_updated();
        au
    }
}
impl Default for Watcher {
    fn default() -> Self {
        Self
    }
}