use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

pub trait Runnable {
    fn run(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Task<'a, T>
where
    T: Runnable,
{
    pub id: Uuid,
    pub name: &'a str,
    pub runnable: T,
}

impl<'a, T> Task<'a, T>
where
    T: Runnable,
{
    pub fn new(name: &'a str, runnable: T) -> Task<'a, T> {
        Task {
            id: Uuid::new_v4(),
            name,
            runnable,
        }
    }
}

impl<'a, T> Runnable for Task<'a, T>
where
    T: Runnable,
{
    fn run(&self) -> Result<(), Error> {
        self.runnable.run()
    }
}
