use clap::Parser;
use config::Config;
use log::info;

use rdo::runner::TaskRunner;
use rdo::script::{load_all_from_config, Script};
use rdo::task::Task;
use rdo::utils::cli::{Cli, Commands};
use rdo::utils::config::{get_config, get_config_from_file, ConfigType};
use rdo::utils::error::Error;
use rdo::utils::logger::setup_logger;

fn main() {
    setup_logger(None);
    let args = Cli::parse();
    handle_command(args).expect("An exception occurred")
}

fn handle_command(args: Cli) -> Result<(), Error> {
    match args.command {
        None => {
            run_scripts(None, None)?;
        }
        Some(command) => match command {
            Commands::Run {
                scripts,
                config: config_path,
                ..
            } => {
                run_scripts(scripts, config_path)?;
            }
            Commands::List {
                config: config_path,
            } => {
                list_scripts(config_path)?;
            }
        },
    }

    Ok(())
}

fn get_config_or_default(config_path: Option<String>) -> Result<Config, Error> {
    match config_path {
        Some(path) => get_config_from_file(&path),
        None => get_config(ConfigType::Production),
    }
}

fn run_scripts(maybe_scripts: Option<String>, config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_from_config(&config)?
        .into_iter()
        .filter(|script| {
            if let Some(scripts_to_run) = &maybe_scripts {
                scripts_to_run.contains(&script.name)
            } else {
                true
            }
        })
        .map(|script| script.into())
        .collect::<Vec<Task<Script>>>();

    info!(
        "Running scripts: {}",
        scripts
            .iter()
            .map(|s| &s.name)
            .fold(String::new(), |acc, s| {
                if acc.is_empty() {
                    s.to_string()
                } else {
                    format!("{}, {}", acc, s)
                }
            })
    );

    let scripts = scripts.iter().collect::<Vec<_>>();
    let task_runner = TaskRunner::new(scripts).unwrap();
    task_runner.run_all().unwrap();

    Ok(())
}

fn list_scripts(config_path: Option<String>) -> Result<(), Error> {
    info!("Listing scripts");
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_from_config(&config).unwrap();

    for script in scripts {
        println!("{}", script.name);
    }

    Ok(())
}
