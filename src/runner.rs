use std::collections::HashSet;
use std::ops::Deref;

use petgraph::graph::{DefaultIx, DiGraph, NodeIndex};
use petgraph::prelude::DfsPostOrder;
use petgraph::visit::{Topo, Walker};

use crate::error::Error;
use crate::runnable::Runnable;
use crate::task::{Task, TaskDependency, TaskId};

pub struct TaskRunner<'a, F>
where
    F: Runnable + 'a,
{
    pub tasks: HashSet<Task<F>>,
    graph: DiGraph<&'a TaskId, TaskDependency, DefaultIx>,
}

impl<'a, F> TaskRunner<'a, F>
where
    F: Runnable + Clone,
{
    pub fn new(tasks: Vec<&'a Task<F>>) -> TaskRunner<'a, F> {
        let mut graph = DiGraph::new();
        for task in tasks.iter() {
            graph.add_node(&task.id);
        }

        TaskRunner {
            tasks: tasks.into_iter().cloned().collect(),
            graph,
        }
    }

    pub fn add_task(&mut self, task: &'a Task<F>) {
        self.tasks.insert(task.clone());
        self.graph.add_node(&task.id);
    }

    pub fn add_tasks(&mut self, tasks: Vec<&'a Task<F>>) {
        for task in tasks {
            self.add_task(task);
        }
    }

    pub fn add_dependency(&mut self, task: &Task<F>, dependency: &Task<F>) -> &mut Self {
        let task_index = self.get_node_index(task.id).unwrap();
        let dependency_index = self.get_node_index(dependency.id).unwrap();
        let edge = TaskDependency::new(task, dependency);
        self.graph.add_edge(task_index, dependency_index, edge);
        self
    }

    fn get_node_index(&self, task_id: TaskId) -> Option<NodeIndex> {
        self.graph.node_indices().find(|index| {
            let node = self.graph.node_weight(*index).unwrap();
            **node == task_id
        })
    }

    pub fn add_dependencies(&mut self, task: &Task<F>, dependencies: Vec<&Task<F>>) -> &mut Self {
        for dependency in dependencies {
            self.add_dependency(task, dependency);
        }

        self
    }

    pub fn run_all(&self) -> Result<(), Error> {
        let tasks = self.tasks.iter().cloned().collect::<Vec<_>>();
        self.run(tasks.iter().collect())
    }

    fn try_run(&self, task_id: TaskId, tasks_run: &mut HashSet<TaskId>) -> Result<(), Error> {
        let dependencies = self
            .graph
            .neighbors_directed(self.get_node_index(task_id).unwrap(), petgraph::Outgoing);

        for dependency in dependencies {
            let dependency = self.graph.node_weight(dependency).unwrap();
            if !tasks_run.contains(dependency) {
                return Err(Error::TaskDependencyNotRun(
                    task_id.0.to_string(),
                    dependency.0.to_string(),
                ));
            }
        }

        let task = self.tasks.iter().find(|task| task.id == task_id).unwrap();
        task.run(tasks_run)?;
        Ok(())
    }

    pub fn run(&self, tasks: Vec<&Task<F>>) -> Result<(), Error> {
        let tasks_to_run = self.get_run_order_ids(tasks)?;
        let mut tasks_run = HashSet::new();
        for task in tasks_to_run {
            self.try_run(task, &mut tasks_run)?;
        }

        Ok(())
    }

    fn get_run_order_ids(&self, tasks: Vec<&Task<F>>) -> Result<Vec<TaskId>, Error> {
        let mut graph = self.graph.clone();
        let deps = self.get_transitive_closure_range(tasks)?;

        let dep_indices = graph
            .node_indices()
            .filter(|index| deps.contains(graph.node_weight(*index).unwrap()))
            .collect::<Vec<_>>();
        graph.retain_nodes(|_, index| dep_indices.contains(&index));

        let mut sorted_tasks = Topo::new(&graph)
            .iter(&graph)
            .map(|node| *graph.node_weight(node).unwrap().deref())
            .collect::<Vec<_>>();

        sorted_tasks.reverse();

        Ok(sorted_tasks)
    }

    pub fn get_run_order(&self, tasks: Vec<&Task<F>>) -> Result<Vec<Task<F>>, Error> {
        let task_ids = self.get_run_order_ids(tasks)?;
        let tasks = task_ids
            .iter()
            .map(|task_id| self.get_task_by_id(*task_id).unwrap())
            .collect::<Vec<_>>();

        Ok(tasks)
    }

    pub fn get_dependencies(&self, task: &Task<F>) -> Result<Vec<Task<F>>, Error> {
        let task_index = self.get_node_index(task.id).unwrap();
        let dependencies = self
            .graph
            .neighbors_directed(task_index, petgraph::Direction::Outgoing)
            .map(|node| *self.graph.node_weight(node).unwrap().deref())
            .map(|task_id| self.get_task_by_id(task_id).unwrap())
            .collect::<Vec<_>>();

        Ok(dependencies)
    }

    fn get_transitive_closure(&self, task: &Task<F>) -> Result<HashSet<TaskId>, Error> {
        let task_index = self.get_node_index(task.id).unwrap();
        let transitive_closure = DfsPostOrder::new(&self.graph, task_index)
            .iter(&self.graph)
            .map(|node| *self.graph.node_weight(node).unwrap().deref())
            .collect::<HashSet<_>>();

        Ok(transitive_closure)
    }

    fn get_transitive_closure_range(&self, tasks: Vec<&Task<F>>) -> Result<HashSet<TaskId>, Error> {
        let mut transitive_closure = HashSet::new();

        for task in tasks {
            let task_closure = self.get_transitive_closure(task)?;
            transitive_closure.extend(task_closure);
        }

        Ok(transitive_closure)
    }

    fn get_task_by_id(&self, task_id: TaskId) -> Result<Task<F>, Error> {
        let task = self.tasks.iter().find(|task| task.id == task_id).cloned();
        match task {
            Some(task) => Ok(task),
            None => Err(Error::TaskNotFound(task_id.0.to_string())),
        }
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

            Task::new(name, runnable)
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
