use std::{cell::RefCell, error::Error};

use serde_json::Value;

pub struct FileWatcher{
    parse: RefCell<Option<Box<dyn Fn(&str) -> Result<Value, Box<dyn Error + Send + Sync>>>>>,
}

impl FileWatcher {
    
}