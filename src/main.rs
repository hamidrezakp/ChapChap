use chrono::NaiveTime;
use clap::Parser;
use config::{Config, File as ConfigFile, FileFormat};
use once_cell::sync::OnceCell;
use psutil::process::ProcessCollector;
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::{env, thread, time::Duration};

#[derive(Debug, Clone, Deserialize, PartialEq)]
enum AppString {
    Plain(String),
    Regex(String),
}

impl AppString {
    fn is_empty(&self) -> bool {
        match self {
            AppString::Plain(s) => s.is_empty(),
            AppString::Regex(s) => s.is_empty(),
        }
    }
}

/// Represent an app in config file
#[derive(Debug, Deserialize)]
struct App {
    name: String,
    enabled: bool,
    slices: Vec<(NaiveTime, NaiveTime)>,
    black_list: bool,
    command: AppString,
    args: AppString,
}

/// Represent an raw_time app in config file
#[derive(Debug, Deserialize)]
struct RawTimeApp {
    name: String,
    enabled: bool,
    slices: Vec<(String, String)>,
    black_list: bool,
    command: String,
    args: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TempApps {
    apps: Vec<RawTimeApp>,
}

#[derive(Parser)]
#[command(name = "Chap Chap", author = "", disable_version_flag=true, about = "simple usage control app", long_about = None, override_usage="chapchap [OPTIONS]")]

struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "./config.toml",
        default_missing_value = "./config.toml",
        help = "configuration file"
    )]
    config: String,
    #[arg(
        short,
        long,
        value_name = "NUMBER",
        default_value = "500",
        default_missing_value = "500",
        help = "delay betwean checking for processes in ms"
    )]
    delay: u64,
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
                command: if app.command.contains("*") {
                    AppString::Regex(app.command.replace("*", ".*"))
                } else {
                    AppString::Plain(app.command.to_owned())
                },
                args: match &app.args {
                    Some(arg) => {
                        if arg.contains("*") {
                            AppString::Regex(arg.replace("*", ".*"))
                        } else {
                            AppString::Plain(arg.to_owned())
                        }
                    }
                    None => AppString::Plain("".to_string()),
                },
            })
            .collect())
    }
}

static RE: OnceCell<HashMap<String, Regex>> = OnceCell::new();

macro_rules! get_regex {
    ($re:expr) => {{
        RE.get()
            .expect("Regex map not initialized")
            .get($re)
            .expect("Failed to retrieve regex from map")
    }};
}

fn main() {
    let mut settings = Config::default();
    let args = Cli::parse();
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
            .merge(ConfigFile::new(&args.config, FileFormat::Toml))
            .expect(&format!("Can't open config file in {}", args.config));
    }

    let apps = settings
        .try_into::<TempApps>()
        .expect("Can't parse Config file")
        .into_app_array()
        .unwrap();

    RE.set(
        apps.iter()
            .map(|app| match (app.command.clone(), app.args.clone()) {
                (AppString::Regex(cmd), AppString::Regex(args)) => [Some(cmd), Some(args)],
                (AppString::Regex(cmd), AppString::Plain(_)) => [Some(cmd), None],
                (AppString::Plain(_), AppString::Regex(args)) => [None, Some(args)],
                _ => [None, None],
            })
            .flatten()
            .flatten()
            .map(|app_str| {
                (
                    app_str.to_owned(),
                    Regex::new(&app_str).expect("Failed to parse regex"),
                )
            })
            .collect(),
    )
    .expect("Failed to create regexes map");

    let mut process_list = ProcessCollector::new().unwrap();

    loop {
        process_list.update().expect("Can't update process list");
        check_apps_and_kill(&apps, &process_list.processes);

        thread::sleep(Duration::from_millis(args.delay));
    }
}

fn check_apps_and_kill(
    apps: &Vec<App>,
    process_list: &BTreeMap<psutil::Pid, psutil::process::Process>,
) {
    let now = chrono::Local::now().time();
    for process in process_list {
        let process = process.1;

        if let Ok(Some(cmd)) = process.cmdline() {
            let cmd = cmd.split(" ").collect::<Vec<&str>>();
            for app in apps {
                if check_eq(&app.command, cmd[0]) {
                    if (app.args.is_empty() || check_eq(&app.args, &cmd[1..].join(" ")))
                        && (app.enabled && kill_or_not(&app, &now))
                    {
                        println!("killing {}", app.name);
                        process.kill().expect("Failed to kill process");
                    }
                }
            }
        }
    }
}

fn check_eq(app_str: &AppString, proc_str: &str) -> bool {
    match app_str {
        AppString::Regex(app_str) => get_regex!(app_str).is_match(proc_str),
        AppString::Plain(app_str) => app_str == proc_str,
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
