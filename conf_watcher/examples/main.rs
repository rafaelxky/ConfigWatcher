use serde::Deserialize;
use std::{thread, time::Duration};
use conf_watcher::{auto_updated::AutoUpdated, watcher::Watcher, file_format::FileFormat};
use conf_watcher::watched_file::WatchedFile;
use conf_watcher::watcher::UpdateType;

#[derive(Deserialize)]
pub struct Config {
    strng: String,
}

fn main() {
    
    println!("Hello, world!");
    let watcher: Watcher = Watcher::new().update_type(UpdateType::Manual);
    let fw: WatchedFile = watcher.watch("file.json").unwrap().json();

    fw.on_modify(||{
        println!("File modified");
    });

    fw.on_access(||{
        println!("File accessed!");
    });

    let json: Config = fw.read_json().unwrap();
    let new_json: AutoUpdated<Config> = fw.auto_update(json);
    
    let au:AutoUpdated<Config>  = fw.manual_updated().unwrap();

    //let au: AutoUpdated<Config> = Watcher::manual_updated_from("file.json", FileFormat::Json).unwrap();
    fw.update();

    loop {
        thread::sleep(Duration::from_millis(500));
        println!("Json - {}", au.get().strng);
        fw.update();
    }
}
