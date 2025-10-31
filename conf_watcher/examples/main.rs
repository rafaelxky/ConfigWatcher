use conf_watcher::{AutoUpdated, Watcher};
use serde::Deserialize;
use std::{thread, time::Duration};

// todo: macro for auto update on file update were the struct itself hold the logic to watch a file
#[derive(Deserialize)]
pub struct Config {
    strng: String,
}

fn main() {
    /*
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
    let new_json: AutoUpdated<Config> = fw.auto_update(json);
    let created_json:AutoUpdated<Config>  = fw.auto_updated().unwrap();
    */

    let au: AutoUpdated<Config> = Watcher::auto_updated_from("file.json").unwrap();

    loop {
        thread::sleep(Duration::from_millis(500));
        println!("Json - {}", au.get().strng);
    }
}
