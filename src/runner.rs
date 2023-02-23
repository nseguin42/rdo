use crate::runnable::Runnable;
use crate::task::Task;
use crate::utils::error::Error;
use crate::utils::graph_binding::GraphBinding;

pub struct TaskRunner<'a, F>
where
    F: Runnable + 'a,
{
    graph_binding: GraphBinding<'a, Task<F>, String>,
}

impl<'a, F> TaskRunner<'a, F>
where
    F: Runnable + Clone,
{
    pub fn new(tasks: Vec<&'a Task<F>>) -> Result<TaskRunner<'a, F>, Error> {
        let graph_binding = GraphBinding::new(tasks)?;
        Ok(TaskRunner { graph_binding })
    }

    pub fn run(&self, tasks: Vec<&Task<F>>) -> Result<(), Error> {
        let sorted_tasks = self.get_run_order(tasks)?;
        for task in sorted_tasks {
            self.run_one_unchecked(task)?;
        }
        Ok(())
    }

    pub fn run_all(&self) -> Result<(), Error> {
        self.run(self.graph_binding.get_all_nodes())
    }

    pub fn get_run_order(
        &'a self,
        tasks: Vec<&'a Task<F>>,
    ) -> Result<impl Iterator<Item = &'a Task<F>>, Error> {
        Ok(self.graph_binding.topological_sort(tasks))
    }

    fn run_one_unchecked<'b>(&'b self, task: &'b Task<F>) -> Result<(), Error> {
        debug!("Running task: {}", task.name);
        task.run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::script::{load_all_from_config, Script};
    use crate::utils::config::{get_config, ConfigType};

    use super::*;

    #[test]
    /// Test that the task runner can correctly resolve dependencies.
    /// Uses a model of this graph:
    /// https://assets.leetcode.com/users/images/63bd7ad6-403c-42f1-b8bb-2ea41e42af9a_1613794080.8115625.png
    fn test_run() {
        let task_factory = |name, deps: Vec<&str>| {
            let runnable = move || {
                println!("Ran Task: {}", name);
                Ok(())
            };

            Task::new(
                name,
                deps.iter().cloned().map(String::from).collect(),
                runnable,
                true,
            )
        };

        let task_1 = task_factory("Task 1", vec!["Task 2", "Task 3"]);
        let task_2 = task_factory("Task 2", vec!["Task 3"]);
        let task_3 = task_factory("Task 3", vec![]);
        let task_4 = task_factory("Task 4", vec!["Task 2", "Task 5", "Task 6"]);
        let task_5 = task_factory("Task 5", vec!["Task 6"]);
        let task_6 = task_factory("Task 6", vec![]);

        let tasks = vec![&task_1, &task_2, &task_3, &task_4, &task_5, &task_6];

        let task_runner = TaskRunner::new(tasks).unwrap();
        task_runner.run_all().unwrap();
    }

    #[test]
    fn test_run_all_scripts() {
        let config = get_config(ConfigType::Test).unwrap();
        let scripts = load_all_from_config(&config)
            .unwrap()
            .into_iter()
            .map(|script| script.into())
            .collect::<Vec<Task<Script>>>();

        // Borrow each script
        let scripts = scripts.iter().collect::<Vec<_>>();

        let runner = TaskRunner::new(scripts).unwrap();
        runner.run_all().unwrap();
    }
}
