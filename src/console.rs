use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::utils::error::Error;

pub struct OutputLine {
    pub text: String,
    pub wrapped_error: Option<Error>,
}

impl OutputLine {
    pub fn new(text: String) -> OutputLine {
        OutputLine {
            text,
            wrapped_error: None,
        }
    }
}

pub struct OutputChannel {
    pub rx: Receiver<OutputLine>,
    pub tx: Sender<OutputLine>,
}

pub async fn create_output_channel() -> OutputChannel {
    let (tx, rx) = mpsc::channel::<OutputLine>(100);
    OutputChannel { rx, tx }
}

pub async fn print_output(mut rx: Receiver<OutputLine>) -> Result<(), Error> {
    while let Some(output_line) = rx.recv().await {
        info!("{}", output_line.text)
    }

    Ok(())
}

pub fn setup_signal_handler() {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Ctrl-C received, exiting");
        std::process::exit(0);
    });
}

pub fn setup_output(rx: Receiver<OutputLine>) {
    tokio::spawn(async move {
        print_output(rx).await.unwrap();
    });
}
