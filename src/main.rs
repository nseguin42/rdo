use clap::Parser;
use rdo::resolver::Resolver;

use rdo::runner::Runner;
use rdo::script::{load_all_from_config, Script};
use rdo::utils::cli::{Cli, Commands};
use rdo::utils::config::get_config_or_default;
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
            run(None, None)?;
        }
        Some(command) => match command {
            Commands::Run {
                scripts,
                config: config_path,
                ..
            } => {
                run(scripts, config_path)?;
            }
            Commands::List {
                config: config_path,
            } => {
                list(config_path)?;
            }
        },
    }

    Ok(())
}

fn run(maybe_script_names: Option<String>, maybe_config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(maybe_config_path)?;
    let scripts = load_all_from_config(&config)?;
    let resolver = Resolver::new(scripts.iter().collect())?;

    let sorted = match maybe_script_names {
        Some(script_names) => {
            let scripts_to_run = script_names
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            resolver.resolve(scripts_to_run)?
        }
        None => resolver.resolve_all()?,
    };

    Runner::<Script>::new(sorted).run()
}

fn list(config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_from_config(&config).unwrap();

    for script in scripts {
        println!("{}", script.name);
    }

    Ok(())
}
