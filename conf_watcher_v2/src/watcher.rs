use std::{cell::RefCell, fs::read_to_string};
use std::error::Error;
use serde_json::Value;

pub enum UpdateType {
    Manual,
    Automatic,
}

pub struct Watcher {
    parse: RefCell<Option<Box<dyn Fn(&str) -> Result<Value, Box<dyn Error + Send + Sync>>>>>,
}

impl Watcher {
    pub fn new() -> Self {
        Self {
            parse: RefCell::new(None),
        }
    }

    pub fn watch<S: ToString>(&self, path: S) /* -> FileWatcher */ {
        let _path = path.to_string();
        // FileWatcher::new(_path, self.parse.clone())   <-- you'll do this eventually
    }

    pub fn json(&self) {
        *self.parse.borrow_mut() = Some(Box::new(|path: &str| -> Result<Value, Box<dyn Error + Send + Sync>> {
            let data = read_to_string(path)?;
            let parsed = serde_json::from_str::<Value>(&data)?;
            Ok(parsed)
        }));
    }
}
