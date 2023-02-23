use crate::runnable::Runnable;

use crate::utils::error::Error;

pub struct Runner<'a, T>
where
    T: Runnable,
{
    tasks: Vec<&'a T>,
}

impl<'a, T> Runner<'a, T>
where
    T: Runnable,
{
    pub fn new(tasks: Vec<&'a T>) -> Self {
        Runner { tasks }
    }

    pub fn run(&self) -> Result<(), Error> {
        for task in &self.tasks {
            task.run()?;
        }
        Ok(())
    }
}
