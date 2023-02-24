use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use petgraph::graph::DiGraph;
use petgraph::prelude::NodeIndex;
use petgraph::visit::{NodeFiltered, Topo, Walker};

use crate::utils::error::Error;

pub trait GraphLike<'a, K> {
    fn get_key(&'a self) -> &'a K;
    fn get_children_keys(&'a self) -> Vec<&'a K>;
}

pub struct GraphBinding<'a, T, K>
where
    T: GraphLike<'a, K> + 'a,
    K: Eq + Hash,
{
    graph: DiGraph<&'a T, ()>,
    key_to_id: HashMap<&'a K, NodeIndex>,
}

impl<'a, T, K> GraphBinding<'a, T, K>
where
    T: GraphLike<'a, K> + Debug + 'a,
    K: Eq + Hash + Debug,
{
    pub fn new(nodes: Vec<&'a T>) -> Result<GraphBinding<'a, T, K>, Error> {
        let graph = DiGraph::new();
        let key_to_id = HashMap::new();
        let mut graph_binding = GraphBinding { graph, key_to_id };
        graph_binding.add_nodes(nodes)?;
        Ok(graph_binding)
    }

    fn add_node(&mut self, node: &'a T) {
        let id = self.graph.add_node(node);
        let result = self.key_to_id.insert(node.get_key(), id);
        if result.is_some() {
            panic!("Duplicate key found: {:?}", node.get_key())
        }
    }

    fn add_nodes(&mut self, nodes: Vec<&'a T>) -> Result<(), Error> {
        for &node in nodes.iter() {
            self.add_node(node);
        }
        self.add_child_edges(nodes)
    }

    fn add_child_edges(&mut self, nodes: Vec<&'a T>) -> Result<(), Error> {
        for node in nodes {
            let node_id = self.find_node_id_by_key(node.get_key())?;
            for child_key in node.get_children_keys() {
                let child_id = self.find_node_id_by_key(child_key)?;
                self.graph.add_edge(child_id, node_id, ());
            }
        }

        Ok(())
    }

    fn find_node_id_by_key(&self, key: &K) -> Result<NodeIndex, Error> {
        let result = self.key_to_id.get(key);
        match result {
            Some(id) => Ok(*id),
            None => Err(Error::Unspecified(format!(
                "Could not find node with key {:?}",
                key
            ))),
        }
    }

    fn get_transitive_closure(&self, nodes: Vec<&'a T>) -> Result<HashSet<NodeIndex>, Error> {
        let mut closure = HashSet::new();
        let mut queue = Vec::new();

        for node in nodes {
            queue.push(node);
        }

        while let Some(node) = queue.pop() {
            let node_id = self.find_node_id_by_key(node.get_key())?;
            if !closure.insert(node_id) {
                continue;
            }

            for child_key in node.get_children_keys() {
                let child_id = self.find_node_id_by_key(child_key)?;
                if !closure.insert(child_id) {
                    continue;
                }
                queue.push(
                    self.graph
                        .node_weight(child_id)
                        .unwrap_or_else(|| panic!("No node at id: {:?}", child_id)),
                );
            }
        }

        Ok(closure)
    }

    pub fn get_all_nodes(&self) -> Vec<&'a T> {
        self.graph.node_weights().copied().collect()
    }

    pub fn find_nodes_by_keys(&self, keys: Vec<K>) -> Result<Vec<&'a T>, Error> {
        let mut nodes = Vec::new();
        for key in keys {
            let node_id = self.find_node_id_by_key(&key)?;
            let node = self.graph.node_weight(node_id).unwrap();
            nodes.push(*node);
        }
        Ok(nodes)
    }

    pub fn topological_sort(&'a self, nodes: Vec<&'a T>) -> impl Iterator<Item = &'a T> + 'a {
        let node_ids = self.get_transitive_closure(nodes).unwrap();
        let node_filter_fn = |id: NodeIndex| node_ids.contains(&id);

        let filtered = NodeFiltered::from_fn(&self.graph, node_filter_fn);
        Topo::new(&filtered).iter(&self.graph).map(|id| {
            *self
                .graph
                .node_weight(id)
                .unwrap_or_else(|| panic!("No node at id: {:?}", id))
        })
    }
}
