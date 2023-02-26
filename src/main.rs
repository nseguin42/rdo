use clap::Parser;
use std::process::exit;

use tokio::spawn;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::watch::Receiver as WatchReceiver;
use tokio::sync::{mpsc, watch};
use tokio::task::spawn_blocking;

use rdo::resolver::Resolver;
use rdo::runnable::Runnable;
use rdo::script::load_all_from_config;
use rdo::utils::cli::{handle_output, handle_signals, read_stdin, Cli, Commands};
use rdo::utils::config::get_config_or_default;
use rdo::utils::logger::setup_logger;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let (stdin_tx, stdin_rx) = watch::channel::<String>(String::new());
    let (stdout_tx, stdout_rx) = mpsc::channel::<String>(100);

    spawn_blocking(move || read_stdin(stdin_tx));
    spawn(handle_signals());
    spawn(handle_output(stdout_rx));

    handle_command(stdin_rx, stdout_tx, args).await;
    exit(0);
}

async fn handle_command(stdin_rx: WatchReceiver<String>, stdout_tx: MpscSender<String>, args: Cli) {
    match args.command {
        None => run(stdin_rx, stdout_tx, None, None).await,
        Some(command) => match command {
            Commands::Run {
                scripts,
                config: config_path,
                ..
            } => run(stdin_rx, stdout_tx, scripts, config_path).await,
            Commands::List {
                config: config_path,
            } => list(config_path),
        },
    }
}

async fn run(
    stdin_rx: WatchReceiver<String>,
    stdout_tx: MpscSender<String>,
    maybe_script_names: Option<String>,
    maybe_config_path: Option<String>,
) {
    let config = get_config_or_default(maybe_config_path).unwrap();
    setup_logger(&config);

    let scripts = load_all_from_config(&config).unwrap();
    let resolver = Resolver::new(scripts.iter().collect()).unwrap();

    let sorted = match maybe_script_names {
        Some(script_names) => {
            let scripts_to_run = script_names
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            resolver.resolve(scripts_to_run).unwrap()
        }
        None => resolver.resolve_all().unwrap(),
    };

    for script in sorted {
        script.run(stdin_rx.clone(), stdout_tx.clone()).await;
    }
}

fn list(config_path: Option<String>) {
    let config = get_config_or_default(config_path).unwrap();
    let scripts = load_all_from_config(&config).unwrap();
    let script_names = scripts
        .iter()
        .map(|s| s.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    println!("Available scripts: {}", script_names);
}
