use clap::Parser;
use log::error;
use tokio::sync::mpsc::Sender;

use rdo::console::{create_output_channel, setup_output, setup_signal_handler, OutputLine};
use rdo::resolver::Resolver;
use rdo::runner::Runner;
use rdo::script::{load_all_from_config, Script};
use rdo::utils::cli::{Cli, Commands};
use rdo::utils::config::get_config_or_default;
use rdo::utils::error::Error;
use rdo::utils::logger::setup_logger;

#[tokio::main]
async fn main() {
    setup_logger(None);
    let output_channel = create_output_channel().await;
    let args = Cli::parse();

    setup_signal_handler();
    setup_output(output_channel.rx);

    if let Err(e) = handle_command(output_channel.tx, args).await {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn handle_command(output: Sender<OutputLine>, args: Cli) -> Result<(), Error> {
    match args.command {
        None => {
            run(output, None, None).await?;
        }
        Some(command) => match command {
            Commands::Run {
                scripts,
                config: config_path,
                ..
            } => {
                run(output, scripts, config_path).await?;
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

async fn run(
    output: Sender<OutputLine>,
    maybe_script_names: Option<String>,
    maybe_config_path: Option<String>,
) -> Result<(), Error> {
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

    Runner::<Script>::new(sorted).run(output).await
}

fn list(config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_from_config(&config).unwrap();

    for script in scripts {
        println!("{}", script.name);
    }

    Ok(())
}
