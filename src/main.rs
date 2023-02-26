use clap::Parser;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::sync::watch::Receiver as WatchReceiver;
use tokio::task;

use rdo::resolver::Resolver;
use rdo::runnable::Runnable;
use rdo::script::load_all_from_config;
use rdo::utils::cli::{Cli, Commands};
use rdo::utils::config::get_config_or_default;
use rdo::utils::error::Error;
use rdo::utils::logger::setup_logger;

fn main() {
    let args = Cli::parse();

    let (stdin_tx, stdin_rx) = watch::channel::<String>(String::new());

    // Use a blocking thread for stdin and and pass it to the async runtime
    std::thread::spawn(move || {
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        loop {
            stdin.read_line(&mut buffer).unwrap();
            stdin_tx.send(buffer.clone()).unwrap();
            buffer.clear();
        }
    });

    Runtime::new()
        .unwrap()
        .block_on(async move { handle_command(stdin_rx, args).await });
}

async fn handle_command(rx: WatchReceiver<String>, args: Cli) {
    match args.command {
        None => run(rx, None, None).await.unwrap(),
        Some(command) => match command {
            Commands::Run {
                scripts,
                config: config_path,
                ..
            } => run(rx, scripts, config_path).await.unwrap(),
            Commands::List {
                config: config_path,
            } => list(config_path).unwrap(),
        },
    }
}

async fn run(
    stdin_rx: WatchReceiver<String>,
    maybe_script_names: Option<String>,
    maybe_config_path: Option<String>,
) -> Result<(), Error> {
    let config = get_config_or_default(maybe_config_path)?;
    setup_logger(&config);

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

    let (output_tx, mut output_rx) = tokio::sync::mpsc::channel(100);

    task::spawn(async move {
        while let Some(line) = output_rx.recv().await {
            println!("{}", line);
        }
    });

    for script in sorted {
        script.run(stdin_rx.clone(), output_tx.clone()).await;
    }

    Ok(())
}

fn list(config_path: Option<String>) -> Result<(), Error> {
    let config = get_config_or_default(config_path)?;
    let scripts = load_all_from_config(&config).unwrap();

    for script in scripts {
        println!("{}", script.name);
    }

    Ok(())
}
