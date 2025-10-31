use notify::{RecommendedWatcher, RecursiveMode, Watcher as nWatcher, event::EventKind};
use serde::de::{DeserializeOwned};

use std::{
    error::Error,
    fs::{self},
    path::Path,
    sync::{Arc, Mutex, MutexGuard, mpsc::channel},
    time::Duration,
};

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
        file_path: T,
    ) -> Result<AutoUpdated<W>, Box<dyn std::error::Error>> {
        let wf = Self::watched_file_from(file_path)?;
        let au: Result<AutoUpdated<W>, Box<dyn Error>> = wf.auto_updated();
        au
    }
}
impl Default for Watcher {
    fn default() -> Self {
        Self
    }
}

type CallbackFuncType = Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>;

#[derive(Clone, Copy)]
pub enum FileFormat {
    Json,
    Yaml,
    Toml,
}

pub struct WatchedFile {
    path: String,
    on_modify: CallbackFuncType,
    on_access: CallbackFuncType,
    format: FileFormat,
}

impl WatchedFile {
    fn new<T: ToString>(file_path: T) -> Result<Self, Box<dyn std::error::Error>> {
        let path_str = file_path.to_string();
        let on_modify = Arc::new(Mutex::new(None));
        let on_access = Arc::new(Mutex::new(None));

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_millis(1000)),
        )?;

        watcher.watch(Path::new(&path_str), RecursiveMode::NonRecursive)?;

        let on_modify_clone = Arc::clone(&on_modify);
        let on_access_clone = Arc::clone(&on_access);

        std::thread::spawn(move || {
            let _watcher = watcher;

            for res in rx {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Modify(_) => {
                            let guard = on_modify_clone.lock().unwrap();
                            if let Some(on_modify) = guard.as_deref() {
                                (on_modify as &(dyn Fn() + Send + 'static))();
                            }
                        }
                        EventKind::Access(_) => {
                            let guard = on_access_clone.lock().unwrap();
                            if let Some(on_access) = guard.as_deref() {
                                (on_access as &(dyn Fn() + Send + 'static))();
                            }
                        }
                        _ => (),
                    }
                } else {
                    continue;
                }
            }
        });

        Ok(WatchedFile {
            path: path_str,
            on_modify,
            on_access,
            format: FileFormat::Json,
        })
    }

    pub fn on_modify<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        *self.on_modify.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn on_access<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        *self.on_access.lock().unwrap() = Some(Box::new(callback));
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn read_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(&self.path)?;
        Ok(data)
    }

    pub fn read_json<T: DeserializeOwned>(&self) -> Result<T, Box<dyn std::error::Error>> {
        let data = self.read_string()?;
        let json = serde_json::from_str(&data)?;
        Ok(json)
    }

    pub fn read_yaml<T: DeserializeOwned>(&self) -> Result<T, Box<dyn std::error::Error>> {
        let data = self.read_string()?;
        let yaml = serde_yaml::from_str(&data)?;
        Ok(yaml)
    }

    pub fn read_toml<T: DeserializeOwned>(&self) -> Result<T, Box<dyn std::error::Error>> {
        let data = self.read_string()?;
        let toml = toml::from_str(&data)?;
        Ok(toml)
    }

    pub fn read<T: DeserializeOwned>(&self) -> Result<T, Box<dyn Error>> {
        let data = fs::read_to_string(&self.path)?;

        let value = match self.format {
            FileFormat::Json => serde_json::from_str(&data)?,
            FileFormat::Yaml => serde_yaml::from_str(&data)?,
            FileFormat::Toml => toml::from_str(&data)?,
        };

        Ok(value)
    }

    pub fn json(mut self) -> Self{
        self.format = FileFormat::Json;
        self
    }  
    pub fn yaml(mut self) -> Self{
        self.format = FileFormat::Yaml;
        self
    }
    pub fn toml(mut self) -> Self{
        self.format = FileFormat::Toml;
        self
    }

    pub fn auto_update_from<T>(&self, target: Arc<Mutex<T>>) -> AutoUpdated<T>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let target_clone = Arc::clone(&target);
        let path = self.path.clone();
        let format = self.format.clone();

        self.on_modify(move || {
            let data = match std::fs::read_to_string(&path) {
                Ok(data) => data,
                Err(err) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Failed to read file {}: {err}", path);
                    return;
                }
            };

           let new: T = match format {
            FileFormat::Json => match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(err) => {
                    //#[cfg(debug_assertions)]
                    //eprintln!("Failed to parse JSON from {}: {err}", path);
                    return;
                }
            },
            FileFormat::Yaml => match serde_yaml::from_str(&data) {
                Ok(v) => v,
                Err(err) => {
                    //#[cfg(debug_assertions)]
                    //eprintln!("Failed to parse YAML from {}: {err}", path);
                    return;
                }
            },
            FileFormat::Toml => match toml::from_str(&data) {
                Ok(v) => v,
                Err(err) => {
                    //#[cfg(debug_assertions)]
                    //eprintln!("Failed to parse TOML from {}: {err}", path);
                    return;
                }
            },
        };

            if let Ok(mut obj) = target_clone.lock() {
                *obj = new;
            } else {
                #[cfg(debug_assertions)]
                eprintln!("Failed to lock shared config object");
            }
        });
        AutoUpdated::wrap(target)
    }

    pub fn auto_update<T>(&self, target: T) -> AutoUpdated<T>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let target = Arc::new(Mutex::new(target));
        self.auto_update_from(target)
    }

    pub fn auto_updated<T>(&self) -> Result<AutoUpdated<T>, Box<dyn Error>>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let target: T = self.read_json()?;
        let au = self.auto_update(target);
        Ok(au)
    }
}

pub trait Reloadable: Sized {
    fn reload_from_str(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>>;
}

// todo: create wrapper class for auto updatable value
pub struct AutoUpdated<T: DeserializeOwned> {
    wrapped: Arc<Mutex<T>>,
}
impl<T: DeserializeOwned> AutoUpdated<T> {
    pub fn new(wrapped: T) -> Self {
        Self {
            wrapped: Arc::new(Mutex::new(wrapped)),
        }
    }
    pub fn wrap(wrapped: Arc<Mutex<T>>) -> Self {
        Self { wrapped }
    }
    pub fn get(&self) -> MutexGuard<'_, T> {
        self.wrapped.lock().expect("Error: lock poisoned!")
    }
    pub fn try_get(&self) -> Option<std::sync::MutexGuard<'_, T>> {
        self.wrapped.try_lock().ok()
    }
    pub fn shared(&self) -> Arc<Mutex<T>> {
        self.wrapped.clone()
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.get();
        f(&*guard)
    }
}

impl<T: std::fmt::Debug + DeserializeOwned> std::fmt::Debug for AutoUpdated<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_get() {
            Some(v) => f.debug_tuple("AutoUpdated").field(&*v).finish(),
            None => f.write_str("AutoUpdated(<locked>)"),
        }
    }
}
