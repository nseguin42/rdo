use std::collections::HashSet;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;
use crate::runnable::Runnable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct TaskId(pub Uuid);

#[derive(Clone)]
pub struct Task<F>
where
    F: Runnable,
{
    pub id: TaskId,
    pub name: String,
    pub runnable: F,
    pub enabled: bool,
}

impl<F> Task<F>
where
    F: Runnable,
{
    pub fn new(name: &str, runnable: F, enabled: bool) -> Task<F> {
        Task {
            id: TaskId(Uuid::new_v4()),
            name: name.to_string(),
            runnable,
            enabled,
        }
    }

    pub fn run<'a>(&'a self, tasks_run: &mut HashSet<&'a Task<F>>) -> Result<(), Error> {
        self.runnable.run()?;
        tasks_run.insert(self);
        Ok(())
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
        self.id == other.id
    }
}

impl<F> Eq for Task<F> where F: Runnable {}

impl<F> std::hash::Hash for Task<F>
where
    F: Runnable,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<F> Debug for Task<F>
where
    F: Runnable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("index", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDependency {
    task: TaskId,
    dependency: TaskId,
}

impl TaskDependency {
    pub fn new<F>(task: &Task<F>, dependency: &Task<F>) -> TaskDependency
    where
        F: Runnable,
    {
        TaskDependency {
            task: task.id,
            dependency: dependency.id,
        }
    }
}
