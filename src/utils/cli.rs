use clap::{Parser, Subcommand};
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;

#[derive(Parser)]
#[command(name = "rdo")]
#[command(bin_name = "rdo")]
#[command(about = "A tool for running scripts with dependencies", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Run the given script(s) and all of their dependencies",
        long_about = "Run the given script(s). If no scripts are given, all scripts will be run."
    )]
    Run {
        #[arg(value_name = "script", long, num_args =..)]
        scripts: Option<String>,
        #[arg(value_name = "config", long)]
        config: Option<String>,
    },

    #[command(about = "List all scripts")]
    List {
        #[arg(value_name = "config", long)]
        config: Option<String>,
    },
}

pub fn read_stdin(stdin_tx: Sender<String>) -> String {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer).unwrap();
        stdin_tx.send(buffer.clone()).unwrap();
        buffer.clear();
    }
}

pub async fn handle_signals() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Received SIGINT, exiting");
    std::process::exit(0);
}

pub async fn handle_output(mut output_rx: Receiver<String>) {
    while let Some(line) = output_rx.recv().await {
        println!("{}", line);
    }
}
