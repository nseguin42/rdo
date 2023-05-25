use clap::{Parser, Subcommand, Args};
use tokio::sync::mpsc::Receiver;
use tokio::sync::watch::Sender;

use crate::utils::error::Error;

#[derive(Debug, Parser)]
#[command(name = "rdo")]
#[command(bin_name = "rdo")]
#[command(about = "A tool for running scripts with dependencies", long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[arg(value_name = "scripts", num_args =..)]
    pub scripts: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    #[arg(short, long, default_value = "config", global = true)]
    pub config: Option<String>,
}

#[derive(Debug, Subcommand, Default)]
pub enum Commands {
    #[command(
    about = "Run the given script(s) and all of their dependencies",
    long_about = "Run the given script(s). If no scripts are given, all scripts will be run."
    )]
    #[default]
    Run,
    #[command(about = "List all scripts")]
    List {
        #[arg(value_name = "config", long)]
        config: Option<String>,
    },
}


pub fn read_stdin(stdin_tx: Sender<String>) -> Result<(), Error> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        let result = stdin_tx.send(buffer.clone());
        match result {
            Ok(_) => (),
            Err(_) => break,
        }

        buffer.clear();
    }

    Err(Error::StdinClosed)
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
