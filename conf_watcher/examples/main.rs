use std::{thread, time::Duration};
use serde::{Deserialize};
use conf_watcher::Watcher;
use conf_watcher_macros::Reloadable;
use conf_watcher::Reloadable;

// todo: macro for auto update on file update were the struct itself hold the logic to watch a file
#[derive(Deserialize, Reloadable)]
pub struct Config{
    strng: String,
}

fn main() {
    println!("Hello, world!");
    let watcher = Watcher::new();
    
    let fw = watcher.watch("file.json").unwrap();
    fw.on_modify(||{
        println!("File modified");
    });
    fw.on_access(||{
        println!("File accessed!");
    });

    let json: Config = fw.read_json().unwrap();
    let new_json = fw.to_auto_update(json);

    loop {
        thread::sleep(Duration::from_millis(500));
        println!("Json - {}", new_json.lock().unwrap().strng);
    }
}
