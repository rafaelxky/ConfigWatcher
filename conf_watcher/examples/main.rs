use serde::Deserialize;
use std::{thread, time::Duration};
use conf_watcher::{auto_updated::AutoUpdated, watcher::Watcher, file_format::FileFormat};

#[derive(Deserialize)]
pub struct Config {
    strng: String,
}

fn main() {
    
    println!("Hello, world!");
    let watcher = Watcher::new();

    let fw = watcher.watch("file.json").unwrap().json();

    fw.on_modify(||{
        println!("File modified");
    });

    fw.on_access(||{
        println!("File accessed!");
    });

    let json: Config = fw.read_json().unwrap();
    let new_json: AutoUpdated<Config> = fw.auto_update(json);
    let au:AutoUpdated<Config>  = fw.auto_updated().unwrap();

    let au: AutoUpdated<Config> = Watcher::auto_updated_from("file.json", FileFormat::Json).unwrap();

    loop {
        thread::sleep(Duration::from_millis(500));
        println!("Json - {}", au.get().strng);
    }
}
