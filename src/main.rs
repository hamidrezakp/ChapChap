use chrono;
use config;
use psutil::process::ProcessCollector;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::{thread, time::Duration};

/// Represent an app in config file
#[derive(Debug, Deserialize)]
struct App {
    enabled: bool,
    slices: Vec<(String, String)>,
    black_list: bool,
    command: String,
}
type Apps = HashMap<String, App>;

fn main() {
    let mut settings = config::Config::default();
    match std::env::var("HOME") {
        Ok(home) => {
            settings
                .merge(config::File::new(
                    &format!("{}/.config/chapcahp/config.toml", home),
                    config::FileFormat::Toml,
                ))
                .unwrap();
        }
        Err(_) => {
            settings
                .merge(config::File::new("config.toml", config::FileFormat::Toml))
                .unwrap();
        }
    }

    let apps = settings.try_into::<Apps>().unwrap();
    let mut process_list = ProcessCollector::new().unwrap();

    loop {
        process_list.update().expect("Can't update process list");
        check_apps_and_kill(&apps, &process_list.processes);

        // going for a short nap (500ms)
        thread::sleep(Duration::from_millis(500));
    }
}

fn check_apps_and_kill(
    apps: &Apps,
    process_list: &BTreeMap<psutil::Pid, psutil::process::Process>,
) {
    let pname_pid: Vec<_> = process_list
        .iter()
        .map(|x| (x.0, x.1.name().unwrap_or("".into())))
        .collect();

    for app in apps {
        if let Some(process) = pname_pid.iter().find(|&x| x.1 == app.1.command) {
            if app.1.enabled && kill_or_not(&app.1) {
                println!("killing {}", app.0);
                std::process::Command::new("kill")
                    .args(&["-9", &process.0.to_string()])
                    .output()
                    .expect(&format!(
                        "failed to kill process {}, with PID {}",
                        &(app.1.command),
                        process.0,
                    ));
            }
        }
    }
}

fn kill_or_not(app: &App) -> bool {
    let now = chrono::Local::now().time();

    // if white_list is on, then we only allow on the app to be run on these time slices
    // otherwise, we must kill the app.
    app.slices
        .iter()
        .map(|range| {
            let start = chrono::NaiveTime::parse_from_str(&range.0, "%H:%M:%S")
                .expect("Time be is format 'HH:MM:SS'");
            let end = chrono::NaiveTime::parse_from_str(&range.1, "%H:%M:%S")
                .expect("Time be is format 'HH:MM:SS'");
            start <= now && now <= end
        })
        .any(|x| x)
        ^ (!app.black_list)
}
