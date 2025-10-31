use notify::{event::EventKind, RecommendedWatcher, RecursiveMode, Watcher as nWatcher};
use std::{
    fs, path::Path, sync::{Arc, Mutex, mpsc::channel}, time::Duration
};
use serde::{Deserialize, de::DeserializeOwned};

pub struct Watcher;

impl Watcher {
    pub fn new() -> Self {
        Watcher
    }

    pub fn watch<T: ToString>(&self, file_path: T) -> Result<WatchedFile, Box<dyn std::error::Error>> {
        WatchedFile::new(file_path)
    }
}

pub struct WatchedFile {
    path: String,
    on_modify: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
    on_access: Arc<Mutex<Option<Box<dyn Fn() + Send + 'static>>>>,
}

impl WatchedFile {
    #[allow(dead_code)]
    fn new<T: ToString>(file_path: T) -> Result<Self, Box<dyn std::error::Error>> {
        let path_str = file_path.to_string();
        let on_modify = Arc::new(Mutex::new(None));
        let on_access= Arc::new(Mutex::new(None));

        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            tx,
            notify::Config::default()
            .with_poll_interval(Duration::from_millis(1000)),
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
                        },
                        EventKind::Access(_) => {
                            let guard = on_access_clone.lock().unwrap();
                            if let Some(on_access) = guard.as_deref() {
                                (on_access as &(dyn Fn() + Send + 'static))();
                            }
                        },
                        _ => (),
                    }
                }
            }
        });

        Ok(WatchedFile {
            path: path_str,
            on_modify,
            on_access,
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

    pub fn read_string (&self) -> Result<String, Box<dyn std::error::Error>> {
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

    pub fn auto_update<T>(&self, target: Arc<Mutex<T>>)
    where
        T: Reloadable + Send + 'static,
    {
        let path = self.path.clone();
        self.on_modify(move || {
            if let Ok(data) = std::fs::read_to_string(&path) {
                let mut obj = target.lock().unwrap();
                if let Err(e) = obj.reload_from_str(&data) {
                    eprintln!("Failed to reload: {}", e);
                }
            }
        });
    }

    pub fn to_auto_update<T>(&self, target: T) -> Arc<Mutex<T>>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let path = self.path.clone();
    let target = Arc::new(Mutex::new(target));
    let target_clone = Arc::clone(&target);

        self.on_modify(move || {
            let data = match std::fs::read_to_string(&path) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to read file {}: {err}", path.display());
                    return;
                }
            };

            let new: T = match serde_json::from_str(&data) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Failed to parse JSON from {}: {err}", path.display());
                    return;
                }
            };

            if let Ok(mut obj) = target_clone.lock() {
                *obj = new;
            } else {
                eprintln!("Failed to lock shared config object");
            }
        });
    target
}

}

pub trait Reloadable: Sized {
    fn reload_from_str(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>>;
}

// todo: create wrapper class for auto updatable value 