use clap::{Parser, Subcommand};

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
