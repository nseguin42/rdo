use std::collections::HashSet;
use std::fmt::Debug;

use crate::error::Error;
use crate::runnable::Runnable;

#[derive(Clone)]
pub struct Task<F>
where
    F: Runnable,
{
    pub name: String,
    pub runnable: F,
    pub dependencies: HashSet<String>,
    pub enabled: bool,
}

impl<F> Task<F>
where
    F: Runnable,
{
    pub fn new(name: &str, dependencies: Vec<String>, runnable: F, enabled: bool) -> Task<F> {
        Task {
            name: name.to_string(),
            dependencies: dependencies.iter().map(|s| s.to_string()).collect(),
            runnable,
            enabled,
        }
    }
}

impl<F> Runnable for Task<F>
where
    F: Runnable,
{
    fn run(&self) -> Result<(), Error> {
        self.runnable.run()
    }
}

impl<F> PartialEq for Task<F>
where
    F: Runnable,
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<F> Eq for Task<F> where F: Runnable {}

impl<F> std::hash::Hash for Task<F>
where
    F: Runnable,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<F> Debug for Task<F>
where
    F: Runnable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task").field("name", &self.name).finish()
    }
}
