use std::collections::HashSet;

use std::process::exit;

use clap::{Parser};
use log::error;
use tokio::spawn;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::watch::Receiver as WatchReceiver;
use tokio::sync::{mpsc, watch};
use tokio::task::spawn_blocking;

use rdo::resolver::Resolver;
use rdo::runnable::Runnable;
use rdo::script::load_all_scripts_from_config;
use rdo::utils::cli::{handle_output, handle_signals, read_stdin, Cli, Commands};
use rdo::utils::config::get_config_or_default;
use rdo::utils::error::Error;
use rdo::utils::logger::setup_logger;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let (stdin_tx, stdin_rx) = watch::channel::<String>(String::new());
    let (stdout_tx, stdout_rx) = mpsc::channel::<String>(100);

    spawn_blocking(move || read_stdin(stdin_tx));
    spawn(handle_signals());
    spawn(handle_output(stdout_rx));

    let result = handle_command(stdin_rx, stdout_tx, args).await;
    match result {
        Ok(_) => exit(0),
        Err(e) => {
            error!("Error: {:?}", e);
            exit(1);
        }
    }
}

async fn handle_command(
    stdin_rx: WatchReceiver<String>,
    stdout_tx: MpscSender<String>,
    args: Cli,
) -> Result<(), Error> {
    match args.command {
        None => run(stdin_rx, stdout_tx, args.scripts, args.global_opts.config).await,
        Some(command) => match command {
            Commands::Run => run(stdin_rx, stdout_tx, args.scripts, args.global_opts.config).await,
            Commands::List {
                config: config_path,
            } => list(config_path),
        },
    }
}

async fn run(
    stdin_rx: WatchReceiver<String>,
    stdout_tx: MpscSender<String>,
    maybe_script_names: Option<Vec<String>>,
    maybe_config_path: Option<String>,
) -> Result<(), Error> {
    let config = get_config_or_default(maybe_config_path)?;
    setup_logger(&config)?;

    let scripts = load_all_scripts_from_config(&config)?;
    let resolver = Resolver::new(scripts.iter().collect())?;

    let sorted = match maybe_script_names {
        Some(script_names) => {
            let scripts_to_run = script_names
                .into_iter()
                .collect::<HashSet<String>>()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            resolver.resolve(scripts_to_run)?
        }
        None => resolver.resolve_all()?,
    };

    for script in sorted {
        if let Err(e) = script.run(stdin_rx.clone(), stdout_tx.clone()).await {
            error!("Error running script {}: {}", script.name, e);
            return Err(e);
        }
    }
    Ok(())
}

fn list(config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_scripts_from_config(&config)?;
    let script_names = scripts
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    println!("Available scripts: {}", script_names);
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncBufReadExt;
    use tokio::select;

    use super::*;

    async fn read_stdin_async(stdin_tx: watch::Sender<String>) {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();
        while let Some(line) = lines.next_line().await.unwrap() {
            stdin_tx.send(line).unwrap();
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_run() {
        let args = "rdo run --config ./config/config.test.toml".split(' ');
        let cli = Cli::parse_from(args);

        let (stdin_tx, stdin_rx) = watch::channel::<String>(String::new());
        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(1);

        select! {
            _ = read_stdin_async(stdin_tx) => {}
            _ = handle_output(stdout_rx) => {}
            result = handle_command(stdin_rx, stdout_tx, cli) => {
                assert!(result.is_ok());
                exit(0);
            }
        }
    }
}
