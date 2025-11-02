use std::fs::File;
use std::{
    error::Error,
};
use serde::de::{DeserializeOwned};
use crate::{auto_updated::AutoUpdated, watched_file::*};
use crate::file_format::FileFormat;
use std::cell::RefCell;
use std::fs::read_to_string;

pub enum UpdateType{
    Manual,
    Automatic,
}

pub struct Watcher{
    update_type: RefCell<UpdateType>,
    read: Option<Box<dyn Fn(&dyn ToString) -> String>>,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher{
            update_type: RefCell::new(UpdateType::Automatic),
            read: None,
        }
    }

    #[cfg(feature = "json")]
    pub fn json(self) -> Self{
        *self.file_format.borrow_mut() = |path: &str| -> String {
            let data = read_to_string(path);
        };
        self
    }

    #[cfg(feature = "yaml")]
    pub fn yaml(self) -> Self{
        *self.file_format.borrow_mut() = FileFormat::Yaml;
        self
    }

    #[cfg(feature = "toml")]
    pub fn toml(self) -> Self{
        *self.file_format.borrow_mut() = FileFormat::Toml;
        self
    }
    pub fn manual(self) -> Self{
        *self.update_type.borrow_mut() = UpdateType::Manual;
        self
    }

    pub fn watch<T: ToString>(
        &self,
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        WatchedFile::new(file_path, self.file_format.borrow(), self.update_type.borrow())
    }

    pub fn watch_manual<T: ToString>(
        &self,
        file_path: T,
    ) -> Result<WatchedFile, Box<dyn std::error::Error>> {
         WatchedFile::new_manual(file_path)
    }

    pub fn watch_automatic<T: ToString>(
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