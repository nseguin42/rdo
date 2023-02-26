use crate::utils::error::Error;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver as WatchReceiver;

#[async_trait]
pub trait Runnable {
    async fn run(
        &self,
        stdin_rx: WatchReceiver<String>,
        output_tx: Sender<String>,
    ) -> Result<(), Error>;
}
