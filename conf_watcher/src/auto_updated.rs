use serde::de::{DeserializeOwned};
use std::{
    sync::{Arc, Mutex, MutexGuard},
};

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