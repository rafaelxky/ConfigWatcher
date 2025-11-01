# ConfWatcher

ConfWatcher allows programmers to easily watch config files (or other files) for updates or access at runtime. 

Start by creating a factory `Watcher`.
Use it to create a `WatchedFile`'.

```rust
use conf_watcher::{Watcher, AutoUpdated};

fn main() {
    // Create a watcher
    let watcher = Watcher::new();

    // Watch a file
    let watched_file = watcher.watch("config.json").unwrap();

    // Create an auto-updated config
    let config: AutoUpdated<MyConfig> = watched_file.auto_updated().unwrap();

    // Access the current config
    {
        let cfg = config.get();
        println!("Current config: {:?}", *cfg);
    }

    // The watcher will automatically update `config` on file changes
}
```

Then you can use `WatchedFile` to trigger functions on update or access.
You can also create an `AutoUpdated<T>` variable wich will automatically update itself based on the config changes 