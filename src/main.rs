use std::collections::BTreeMap;
use std::{env, thread, time::Duration};

use chrono::NaiveTime;
use clap::Parser;
use psutil::process::ProcessCollector;
use regex::Regex;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct Config {
    apps: Vec<App>,
    #[serde(default = "default_delay")]
    delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AppString {
    Plain(String),
    Regex(#[serde(deserialize_with = "deserialize_regex")] Regex),
}

/// Represent an app in config file
#[derive(Debug, Deserialize)]
struct App {
    name: String,
    enabled: bool,
    slices: Vec<(NaiveTime, NaiveTime)>,
    black_list: bool,
    command: AppString,
    args: Option<AppString>,
}

#[derive(Parser)]
#[command(
    name = "Chap Chap",
    author = "",
    disable_version_flag=true,
    about = "Kill distracting apps",
    long_about = None,
    override_usage="chapchap [OPTIONS]")
]
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
}

fn main() -> Result<(), String> {
    let mut config = config::Config::default();
    let args = Cli::parse();
    if let Ok(config_file_path) = env::var("XDG_CONFIG_HOME") {
        config
            .merge(config::File::new(
                &format!("{config_file_path}/chapchap/config.toml",),
                config::FileFormat::Toml,
            ))
            .map_err(|e| {
                format!("Can't open config file in {config_file_path}/chapchap/config.toml: {e:?}",)
            })?;
    // Fallback to search config file in CWD
    } else {
        config
            .merge(config::File::new(&args.config, config::FileFormat::Toml))
            .map_err(|e| format!("Can't open config file in {}: {e:?}", args.config))?;
    }

    let config = config
        .try_into::<Config>()
        .map_err(|e| format!("Can't parse configuration: {e:?}"))?;
    let apps = config.apps;
    let delay = Duration::from_millis(config.delay_ms);

    let mut process_list =
        ProcessCollector::new().map_err(|e| format!("failed to get process list: {e:?}"))?;

    loop {
        if let Err(e) = process_list.update() {
            eprintln!("Updating process list failed: {e:?}");
        }

        check_apps_and_kill(&apps, &process_list.processes);
        thread::sleep(delay);
    }
}

fn check_apps_and_kill(
    apps: &Vec<App>,
    process_list: &BTreeMap<psutil::Pid, psutil::process::Process>,
) {
    let now = chrono::Local::now().time();
    for (_, process) in process_list {
        if let Ok(Some(cmd)) = process.cmdline() {
            let cmd = cmd.split(" ").collect::<Vec<&str>>();
            for app in apps {
                if check_eq(&app.command, cmd[0])
                    && (app.args.is_none()
                        || app
                            .args
                            .as_ref()
                            .is_some_and(|args| check_eq(&args, &cmd[1..].join(" "))))
                    && (app.enabled && should_kill(&app, &now))
                {
                    println!("killing {}", app.name);
                    if let Err(e) = process.kill() {
                        eprintln!("error while trying to kill {}: {e:?}", app.name)
                    }
                }
            }
        }
    }
}

fn check_eq(app_str: &AppString, proc_str: &str) -> bool {
    if proc_str.starts_with("telegram") {
        println!("{app_str:?}, {proc_str}");
    }
    match app_str {
        AppString::Regex(re) => re.is_match(proc_str),
        AppString::Plain(app_str) => app_str == proc_str,
    }
}

fn should_kill(app: &App, now: &NaiveTime) -> bool {
    // if white_list is on, then we only allow on the app to be run on these time slices
    // otherwise, we must kill the app.
    app.slices
        .iter()
        .map(|range| &range.0 <= now && now <= &range.1)
        .any(|x| x)
        ^ (!app.black_list)
}

fn deserialize_regex<'de, D>(d: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    Regex::new(&s).map_err(|e| serde::de::Error::custom(&format!("invalid regex: {e:?}")))
}

const fn default_delay() -> u64 {
    500
}
