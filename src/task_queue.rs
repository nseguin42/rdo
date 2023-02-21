use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

use petgraph::graph::DiGraph;
use petgraph::prelude::NodeIndex;
use petgraph::visit::{NodeFiltered, Topo, Walker};

use crate::runnable::Runnable;
use crate::task::Task;

pub trait GraphLike<'a, K>
where
    K: PartialEq + Eq + Hash,
{
    fn get_key(&'a self) -> &'a K;
    fn get_children_keys(&'a self) -> Vec<&'a K>;
}

impl<'a, F> GraphLike<'a, String> for Task<F>
where
    F: Runnable + 'a,
{
    fn get_key(&'a self) -> &'a String {
        &self.name
    }

    fn get_children_keys(&'a self) -> Vec<&'a String> {
        self.dependencies.iter().collect()
    }
}

pub struct GraphBinding<'a, T, K>
where
    T: GraphLike<'a, K> + PartialEq + Eq + 'a,
    K: PartialEq + Eq + Hash,
{
    graph: DiGraph<&'a T, ()>,
    key_to_id: HashMap<&'a K, NodeIndex>,
}

impl<'a, T, K> GraphBinding<'a, T, K>
where
    T: GraphLike<'a, K> + PartialEq + Eq + 'a,
    K: PartialEq + Eq + Hash,
{
    pub fn new(nodes: Vec<&'a T>) -> GraphBinding<'a, T, K> {
        let graph = DiGraph::new();
        let key_to_id = HashMap::new();
        let mut graph_binding = GraphBinding { graph, key_to_id };
        graph_binding.add_nodes(nodes);
        graph_binding
    }

    fn add_node(&mut self, node: &'a T) {
        let id = self.graph.add_node(node);
        self.key_to_id.insert(node.get_key(), id);
    }

    fn add_nodes(&mut self, nodes: Vec<&'a T>) {
        for &node in nodes.iter() {
            self.add_node(node);
        }
        self.add_child_edges(nodes);
    }

    fn add_child_edges(&mut self, nodes: Vec<&'a T>) {
        for node in nodes {
            let node_id = self.key_to_id.get(node.get_key()).unwrap();
            for child_key in node.get_children_keys() {
                let child_id = self.key_to_id.get(child_key).unwrap();
                self.graph.add_edge(*child_id, *node_id, ());
            }
        }
    }

    fn get_transitive_closure(&'a self, nodes: Vec<&'a T>) -> HashSet<&NodeIndex> {
        let mut closure = HashSet::new();
        let mut queue = VecDeque::new();

        for node in nodes {
            queue.push_back(node);
        }

        while let Some(node) = queue.pop_front() {
            for child_key in node.get_children_keys() {
                let child_id = self.key_to_id.get(child_key).unwrap();
                if !closure.insert(child_id) {
                    continue;
                }
                queue.push_back(self.graph.node_weight(*child_id).unwrap());
            }
        }

        closure
    }

    pub fn get_all_nodes(&'a self) -> Vec<&'a T> {
        self.graph.node_weights().copied().collect()
    }

    pub fn topological_sort(&'a self, nodes: Vec<&'a T>) -> impl Iterator<Item = &'a T> + 'a {
        let node_ids = self.get_transitive_closure(nodes);
        let node_filter_fn = |id: NodeIndex| node_ids.contains(&id);

        let filtered = NodeFiltered::from_fn(&self.graph, node_filter_fn);
        Topo::new(&filtered)
            .iter(&self.graph)
            .map(|id| *self.graph.node_weight(id).unwrap())
    }
}
