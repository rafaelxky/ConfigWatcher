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
    
    let watcher: Watcher = Watcher::new().update_type(UpdateType::Manual);
    let fw: WatchedFile = watcher.watch("file.json").unwrap().json();
    let au:AutoUpdated<Config>  = fw.manual_updated().unwrap();

    loop {
        thread::sleep(Duration::from_millis(500));
        println!("Json - {}", au.get().strng);
        fw.update();
    }
}
