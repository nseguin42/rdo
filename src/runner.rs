use std::collections::HashSet;

use petgraph::graph::{DefaultIx, DiGraph, NodeIndex};
use petgraph::prelude::Dfs;
use petgraph::visit::{Topo, Visitable, Walker};

use crate::error::Error;
use crate::runnable::Runnable;
use crate::task::{Task, TaskDependency};

pub struct TaskRunner<'a, F>
where
    F: Runnable + 'a,
{
    pub tasks: HashSet<Task<F>>,
    graph: DiGraph<&'a Task<F>, TaskDependency, DefaultIx>,
}

impl<'a, F> TaskRunner<'a, F>
where
    F: Runnable + Clone,
{
    pub fn new(tasks: Vec<&'a Task<F>>) -> TaskRunner<'a, F> {
        let mut graph = DiGraph::new();
        for task in tasks.iter() {
            graph.add_node(*task);
        }

        TaskRunner {
            tasks: tasks.into_iter().cloned().collect(),
            graph,
        }
    }

    pub fn add_task(&mut self, task: &'a Task<F>) {
        self.tasks.insert(task.clone());
        self.graph.add_node(task);
    }

    pub fn add_tasks(&mut self, tasks: Vec<&'a Task<F>>) {
        for task in tasks {
            self.add_task(task);
        }
    }

    pub fn add_dependency(&mut self, task: &Task<F>, dependency: &Task<F>) -> &mut Self {
        let task_index = self.get_node_index(task).unwrap();
        let dependency_index = self.get_node_index(dependency).unwrap();
        let edge = TaskDependency::new(task, dependency);
        self.graph.add_edge(task_index, dependency_index, edge);
        self
    }

    fn get_node_index(&self, task: &Task<F>) -> Option<NodeIndex> {
        self.graph.node_indices().find(|index| {
            let &node = self.graph.node_weight(*index).unwrap();
            node == task
        })
    }

    pub fn add_dependencies(&mut self, task: &Task<F>, dependencies: Vec<&Task<F>>) -> &mut Self {
        for dependency in dependencies {
            self.add_dependency(task, dependency);
        }

        self
    }

    pub fn run_all(&self) -> Result<(), Error> {
        let tasks = self.tasks.iter().filter(|t| t.enabled);
        self.run(tasks.into_iter().collect())
    }

    fn try_run<'b>(
        &'b self,
        task: &Task<F>,
        tasks_run: &mut HashSet<&'b Task<F>>,
    ) -> Result<(), Error> {
        let dependencies = self
            .graph
            .neighbors_directed(self.get_node_index(task).unwrap(), petgraph::Outgoing);

        for dependency in dependencies {
            let dependency = self.graph.node_weight(dependency).unwrap();
            if !tasks_run.contains(dependency) {
                return Err(Error::TaskDependencyNotRun(
                    task.name.clone(),
                    dependency.name.clone(),
                ));
            }
        }

        let task = self.tasks.iter().find(|t| *t == task).unwrap();

        if !task.enabled {
            return Err(Error::Task("Task is disabled".to_string()));
        }

        task.run(tasks_run)?;
        Ok(())
    }

    pub fn run(&self, tasks: Vec<&Task<F>>) -> Result<(), Error> {
        let tasks_to_run = self.get_run_order(tasks)?.into_iter().filter(|t| t.enabled);

        let mut tasks_run = HashSet::new();
        for task in tasks_to_run {
            self.try_run(task, &mut tasks_run)?;
        }

        Ok(())
    }

    /// Perform a depth first search on each node individually, reusing the visit
    /// map for each node.
    fn get_transitive_closure(&self, nodes: Vec<NodeIndex>) -> Vec<NodeIndex> {
        let dfs = Dfs::from_parts(nodes, self.graph.visit_map());
        dfs.iter(&self.graph).collect()
    }

    pub fn get_run_order(&self, tasks: Vec<&Task<F>>) -> Result<Vec<&Task<F>>, Error> {
        let mut graph = self.graph.clone();
        let nodes = tasks
            .iter()
            .map(|task| self.get_node_index(task).unwrap())
            .collect::<Vec<_>>();

        let deps = self.get_transitive_closure(nodes);
        graph.retain_nodes(|_, index| deps.contains(&index));

        let mut sorted_tasks = Topo::new(&graph)
            .iter(&graph)
            .map(|node| *graph.node_weight(node).unwrap())
            .collect::<Vec<_>>();

        sorted_tasks.reverse();

        Ok(sorted_tasks)
    }
    pub fn get_task_dependencies(&self, task: &Task<F>) -> Result<Vec<&Task<F>>, Error> {
        let task_index = self.get_node_index(task).unwrap();
        let dependencies = self
            .graph
            .neighbors_directed(task_index, petgraph::Direction::Outgoing)
            .map(|node| *self.graph.node_weight(node).unwrap())
            .collect::<Vec<_>>();

        Ok(dependencies)
    }

    pub fn get_task_by_name(&self, task_name: String) -> Result<&Task<F>, Error> {
        let task = self
            .tasks
            .iter()
            .find(|task| task.name == task_name)
            .cloned()
            .map(|task| Box::leak(Box::new(task)));

        match task {
            Some(task) => Ok(task),
            None => Err(Error::TaskNotFound(task_name)),
        }
    }

    pub fn add_dependency_by_name(
        &mut self,
        task_name: String,
        dependency_name: String,
    ) -> Result<(), Error> {
        let task = self.get_task_by_name(task_name)?.clone();
        let dependency = self.get_task_by_name(dependency_name)?.clone();
        self.add_dependency(&task, &dependency);

        Ok(())
    }
}

pub trait WithDependencies: Runnable {
    fn get_dependencies(&self) -> Vec<String>;
}

impl<'a, F> TaskRunner<'a, F>
where
    F: WithDependencies + Clone,
{
    pub fn new_with_dependencies(tasks: Vec<&'a Task<F>>) -> TaskRunner<'a, F> {
        let mut task_runner = Self::new(tasks);
        let task_names = task_runner
            .tasks
            .iter()
            .map(|task| task.name.clone())
            .collect::<Vec<_>>();

        for task_name in task_names {
            let task = task_runner.get_task_by_name(task_name.clone()).unwrap();
            let dependencies = task.runnable.get_dependencies();
            dbg!(task_name.clone(), dependencies.clone());
            for dependency_name in dependencies {
                task_runner
                    .add_dependency_by_name(task_name.clone(), dependency_name)
                    .unwrap();
            }
        }

        task_runner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Test that the task runner can correctly resolve dependencies.
    /// Uses a model of this graph:
    /// https://assets.leetcode.com/users/images/63bd7ad6-403c-42f1-b8bb-2ea41e42af9a_1613794080.8115625.png
    fn test_run() {
        let task_factory = |name| {
            let runnable = move || {
                println!("Ran Task: {}", name);
                Ok(())
            };

            Task::new(name, runnable, true)
        };

        let task_1 = task_factory("Task 1");
        let task_2 = task_factory("Task 2");
        let task_3 = task_factory("Task 3");
        let task_4 = task_factory("Task 4");
        let task_5 = task_factory("Task 5");
        let task_6 = task_factory("Task 6");

        let tasks = vec![&task_1, &task_2, &task_3, &task_4, &task_5, &task_6];
        let mut task_runner = TaskRunner::new(tasks);

        task_runner
            .add_dependencies(&task_1, vec![&task_2, &task_3])
            .add_dependency(&task_2, &task_3)
            .add_dependencies(&task_4, vec![&task_2, &task_5, &task_6])
            .add_dependency(&task_5, &task_6);

        task_runner.run_all().unwrap();
    }
}
