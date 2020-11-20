use chrono::NaiveTime;
use config::{Config, File as ConfigFile, FileFormat};
use psutil::process::ProcessCollector;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::{env, process::Command, thread, time::Duration};

/// Represent an app in config file
#[derive(Debug, Deserialize)]
struct App {
    name: String,
    enabled: bool,
    slices: Vec<(NaiveTime, NaiveTime)>,
    black_list: bool,
    command: String,
}

/// Represent an raw_time app in config file
#[derive(Debug, Deserialize)]
struct RawTimeApp {
    name: String,
    enabled: bool,
    slices: Vec<(String, String)>,
    black_list: bool,
    command: String,
}

#[derive(Debug, Deserialize)]
struct TempApps {
    apps: Vec<RawTimeApp>,
}

impl TempApps {
    pub fn into_app_array(self: Self) -> Result<Vec<App>, &'static str> {
        let parse_time = |time_str, app_name| {
            NaiveTime::parse_from_str(time_str, "%H:%M:%S").expect(&format!(
                "Syntax error, can't parse time: '{}' in app '{}'",
                time_str, app_name
            ))
        };

        Ok(self
            .apps
            .iter()
            .map(|app| App {
                name: app.name.to_owned(),
                enabled: app.enabled,
                slices: app
                    .slices
                    .iter()
                    .map(|slice| {
                        (
                            parse_time(&slice.0, &app.name),
                            parse_time(&slice.1, &app.name),
                        )
                    })
                    .collect(),
                black_list: app.black_list,
                command: app.command.to_owned(),
            })
            .collect())
    }
}

fn main() {
    let mut settings = Config::default();
    if let Ok(config_file_path) = env::var("XDG_CONFIG_HOME") {
        settings
            .merge(ConfigFile::new(
                &format!("{}/chapchap/config.toml", config_file_path),
                FileFormat::Toml,
            ))
            .expect(&format!(
                "Can't open config file in {}/chapchap/config.toml",
                config_file_path
            ));
    // Fallback to search config file in CWD
    } else {
        settings
            .merge(ConfigFile::new("config.toml", FileFormat::Toml))
            .expect("Can't open config file in current working directory");
    }

    let apps = settings
        .try_into::<TempApps>()
        .expect("Can't parse Config file")
        .into_app_array()
        .unwrap();

    let mut process_list = ProcessCollector::new().unwrap();

    loop {
        process_list.update().expect("Can't update process list");
        check_apps_and_kill(&apps, &process_list.processes);

        // going for a short nap (500ms)
        thread::sleep(Duration::from_millis(500));
    }
}

fn check_apps_and_kill(
    apps: &Vec<App>,
    process_list: &BTreeMap<psutil::Pid, psutil::process::Process>,
) {
    let pname_pid: Vec<_> = process_list
        .iter()
        .map(|x| (x.0, x.1.name().unwrap_or("".into())))
        .collect();

    let now = chrono::Local::now().time();

    for app in apps {
        if let Some(process) = pname_pid.iter().find(|&p| p.1 == app.command) {
            if app.enabled && kill_or_not(&app, &now) {
                println!("killing {}", app.name);
                Command::new("kill")
                    .args(&["-9", &process.0.to_string()])
                    .output()
                    .expect(&format!(
                        "failed to kill process {}, with PID {}",
                        &(app.command),
                        process.0,
                    ));
            }
        }
    }
}

fn kill_or_not(app: &App, now: &NaiveTime) -> bool {
    // if white_list is on, then we only allow on the app to be run on these time slices
    // otherwise, we must kill the app.
    app.slices
        .iter()
        .map(|range| &range.0 <= now && now <= &range.1)
        .any(|x| x)
        ^ (!app.black_list)
}
