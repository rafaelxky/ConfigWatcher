use std::{
    error::Error,
};
use serde::de::{DeserializeOwned};
use crate::{auto_updated::AutoUpdated, watched_file::*};
use crate::file_format::FileFormat;
use std::cell::RefCell;

pub enum UpdateType{
    Manual,
    Automatic,
}

pub struct Watcher{
    update_type: RefCell<UpdateType>,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher{
            update_type: RefCell::new(UpdateType::Automatic),
        }
    }

    pub fn update_type(self ,update_type: UpdateType) -> Self {
        *self.update_type.borrow_mut() = update_type;
        self
    }

    pub fn watch<T: ToString>(
        &self,
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        match *self.update_type.borrow() {
            UpdateType::Manual => WatchedFile::new(file_path),
            UpdateType::Automatic => WatchedFile::new_manual(file_path),
        }
    }

    pub fn watched_file_from<T: ToString>(
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        Self::new().watch(file_path)
    }

    pub fn auto_updated_from<T: ToString, W: DeserializeOwned + Send + 'static>(
        file_path: T, file_format: FileFormat
    ) -> Result<(AutoUpdated<W>, WatchedFile), Box<dyn std::error::Error>> {
        let watched_file: WatchedFile = WatchedFile::new(file_path)?.format(file_format);
        let auto_updated_value: Result<AutoUpdated<W>, Box<dyn Error>> = watched_file.auto_updated();
        match auto_updated_value {
            Ok(value) => Ok((value, watched_file)),
            Err(e) => Err(e),
        }
    }

    pub fn manual_updated_from<T: ToString, W: DeserializeOwned + Send + 'static>(
        file_path: T, file_format: FileFormat
    ) -> Result<(AutoUpdated<W>, WatchedFile), Box<dyn std::error::Error>> {
        let watched_file: WatchedFile = WatchedFile::new_manual(file_path)?.format(file_format);
        let auto_updated_value: Result<AutoUpdated<W>, Box<dyn Error>> = watched_file.auto_updated();
        match auto_updated_value {
            Ok(value) => Ok((value, watched_file)),
            Err(e) => Err(e),
        }
    }

}
impl Default for Watcher {
    fn default() -> Self {
        Self {
            update_type: RefCell::new(UpdateType::Automatic),
        }
    }
}