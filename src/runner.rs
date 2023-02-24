use tokio::sync::mpsc::Sender;

use crate::console::OutputLine;
use crate::runnable::Runnable;
use crate::script::Script;
use crate::utils::error::Error;

pub struct Runner<'a, T>
where
    T: Runnable,
{
    tasks: Vec<&'a T>,
}

impl<'a> Runner<'a, Script> {
    pub fn new(tasks: Vec<&'a Script>) -> Runner<'a, Script> {
        Runner { tasks }
    }

    pub async fn run(&self, output: Sender<OutputLine>) -> Result<(), Error> {
        for script in self.tasks.iter() {
            let msg = format!("Running script: {}", script.name);
            output.send(OutputLine::new(msg)).await?;
            script.run()?;
        }
        Ok(())
    }
}
