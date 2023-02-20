use crate::error::Error;

pub trait Runnable {
    fn run(&self) -> Result<(), Error>;
}

impl<F> Runnable for F
where
    F: Fn() -> Result<(), Error> + Send + Sync,
{
    fn run(&self) -> Result<(), Error> {
        self()
    }
}

impl Runnable for () {
    fn run(&self) -> Result<(), Error> {
        Ok(())
    }
}
