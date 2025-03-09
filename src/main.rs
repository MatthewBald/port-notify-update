use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{env, fs, path::Path};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = env::var("FILEPATH").expect("FILEPATH environment variable should be set");

    log::info!("Watching {path}");
    
    match fs::exists(format!("{path}/forwarded_port")) {
        Ok(exists) => {
            if exists {
                log::info!("Posting initial contents");
                post_port(&read_file(&format!("{path}/forwarded_port")));
            }
        },
        Err(_) => log::info!("Did not find existing file. Will continue to watch the directory."),
    }

    if let Err(error) = watch(path) {
        log::error!("Error: {error:?}");
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                let path = event.paths[0].display().to_string();
                if path.contains("forwarded_port") {
                    change_event_handler(event)
                }
                else {
                    log::info!("Detected an unrelated file change event {event:?}")
                }
            },
            Err(error) => log::error!("Error! {error:?}"),
        }
    }

    Ok(())
}

fn change_event_handler(event: Event) {
    match event.kind {
        notify::EventKind::Access(access_kind) => {
            match access_kind {
                notify::event::AccessKind::Close(access_mode) => {
                    match access_mode {
                        notify::event::AccessMode::Write => handle_file_saved(event),
                        _ => log_no_change(event),
                    }
                },
                _ => log_no_change(event),
            }
        },
        notify::EventKind::Create(create_kind) => {
            match create_kind {
                notify::event::CreateKind::File => handle_file_saved(event),
                _ => log_no_change(event),
            }
        },
        _ => log_no_change(event),
    }
}

fn log_no_change(event: Event) {
    log::info!("Not handling {event:?}");
}

fn handle_file_saved(event: Event) {
    let path = event.paths[0].display().to_string();
    post_port(&read_file(&path));
}

fn read_file(path: &String) -> u32 {
    log::info!("Reading contents of file {path}");
    let contents = fs::read_to_string(path)
        .expect("Should have been able to read the file");

    log::info!("Read '{contents}' from file");

    match contents.parse() {
        Ok(number) => return number,
        Err(error) => {
            log::error!("{error:?}");
            return 0;
        },
    }
}

fn post_port(port: &u32) {
    if *port <= 0 {
        return;
    }

    let body = format!("json={{\"listen_port\": {port}}}");

    let base_url = env::var("BASEURL").expect("BASEURL environment variable should be set");
    let client = reqwest::blocking::Client::new();
    let res = client.post(format!("{base_url}/api/v2/app/setPreferences"))
        .body(body)
        .header("content-type", "application/x-www-form-urlencoded")
        .send();

    match res {
        Ok(_) => log::info!("OK ({res:?})"),
        Err(_) => log::error!("ERR ({res:?})"),
    }
}