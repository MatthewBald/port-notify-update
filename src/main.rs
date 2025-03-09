use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{env, fs, path::Path};
use log::{debug, error, info};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let path = env::var("FILEPATH").expect("FILEPATH environment variable should be set");

    info!("Watching {path}");

    match fs::exists(format!("{path}/forwarded_port")) {
        Ok(exists) => {
            if exists {
                info!("Found a forwarded_port file, posting its initial contents");
                post_port(&read_file(&format!("{path}/forwarded_port")));
            }
        }
        Err(_) => info!("Did not find existing forwarded_port file. Will continue to watch the directory."),
    }

    if let Err(error) = watch(path) {
        error!("{error:?}");
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
                } else {
                    debug!("Detected an unrelated file change event {event:?}")
                }
            }
            Err(error) => error!("Error! {error:?}"),
        }
    }

    Ok(())
}

fn change_event_handler(event: Event) {
    match event.kind {
        notify::EventKind::Access(access_kind) => match access_kind {
            notify::event::AccessKind::Close(access_mode) => match access_mode {
                notify::event::AccessMode::Write => handle_file_saved(event),
                _ => log_no_change(event),
            },
            _ => log_no_change(event),
        },
        notify::EventKind::Create(create_kind) => match create_kind {
            notify::event::CreateKind::File => handle_file_saved(event),
            _ => log_no_change(event),
        },
        _ => log_no_change(event),
    }
}

fn log_no_change(event: Event) {
    debug!("Not handling {event:?}");
}

fn handle_file_saved(event: Event) {
    let path = event.paths[0].display().to_string();
    post_port(&read_file(&path));
}

fn read_file(path: &String) -> u32 {
    debug!("Reading contents of file {path}");
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    debug!("Read '{contents}' from file");

    match contents.parse() {
        Ok(number) => return number,
        Err(error) => {
            error!("{error:?}");
            return 0;
        }
    }
}

fn post_port(port: &u32) {
    if *port <= 0 {
        return;
    }

    let client = reqwest::blocking::Client::new();
    let base_url = env::var("BASEURL").expect("BASEURL environment variable should be set");
    let body = format!("json={{\"listen_port\": {port}}}");
    let res = client
        .post(format!("{base_url}/api/v2/app/setPreferences"))
        .body(body)
        .header("content-type", "application/x-www-form-urlencoded")
        .send();

    match res {
        Ok(_) => info!("OK ({res:?})"),
        Err(_) => error!("ERR ({res:?})"),
    }
}
