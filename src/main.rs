use clap::Parser;
use log::info;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender as MpscSender;
use tokio::sync::watch::{Receiver as WatchReceiver, Sender};
use tokio::sync::{mpsc, watch};
use tokio::task;

use rdo::resolver::Resolver;
use rdo::runnable::Runnable;
use rdo::script::load_all_from_config;
use rdo::utils::cli::{Cli, Commands};
use rdo::utils::config::get_config_or_default;
use rdo::utils::logger::setup_logger;

fn main() {
    let args = Cli::parse();

    let (stdin_tx, stdin_rx) = watch::channel::<String>(String::new());
    let (stdout_tx, stdout_rx) = mpsc::channel::<String>(100);

    std::thread::spawn(move || read_stdin(stdin_tx));

    Runtime::new().unwrap().block_on(async move {
        task::spawn(handle_signals());
        task::spawn(handle_output(stdout_rx));
        handle_command(stdin_rx, stdout_tx, args).await;
    });
}

fn read_stdin(stdin_tx: Sender<String>) -> String {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer).unwrap();
        stdin_tx.send(buffer.clone()).unwrap();
        buffer.clear();
    }
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

async fn handle_signals() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Received SIGINT, exiting");
    std::process::exit(0);
}

async fn handle_output(mut output_rx: tokio::sync::mpsc::Receiver<String>) {
    while let Some(line) = output_rx.recv().await {
        println!("{}", line);
    }
}
