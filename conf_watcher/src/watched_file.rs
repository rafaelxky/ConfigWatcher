use notify::{RecommendedWatcher, RecursiveMode, Watcher as nWatcher, event::EventKind};
use serde::de::{DeserializeOwned};
use std::{
    error::Error,
    fs::{self},
    path::Path,
    sync::{Arc, Mutex, mpsc::channel},
    time::Duration,
};
use crate::{auto_updated::AutoUpdated, file_format::FileFormat};

type CallbackFuncType = Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>;

pub struct WatchedFile {
    path: String,
    on_modify: CallbackFuncType,
    on_access: CallbackFuncType,
    format: FileFormat,
}

impl WatchedFile {
    pub fn new<T: ToString>(file_path: T) -> Result<Self, Box<dyn std::error::Error>> {
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
    pub fn format(mut self, format: FileFormat) -> Self{
        self.format = format;
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
                Err(_err) => {
                    //#[cfg(debug_assertions)]
                    //eprintln!("Failed to parse JSON from {}: {err}", path);
                    return;
                }
            },
            FileFormat::Yaml => match serde_yaml::from_str(&data) {
                Ok(v) => v,
                Err(_err) => {
                    //#[cfg(debug_assertions)]
                    //eprintln!("Failed to parse YAML from {}: {err}", path);
                    return;
                }
            },
            FileFormat::Toml => match toml::from_str(&data) {
                Ok(v) => v,
                Err(_err) => {
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
        let target: T = self.read()?;
        let au = self.auto_update(target);
        Ok(au)
    }
}
