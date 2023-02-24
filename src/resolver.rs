use std::fmt::Debug;
use std::hash::Hash;

use crate::utils::error::Error;
use crate::utils::graph_binding::{GraphBinding, GraphLike};

pub struct Resolver<'a, T, K>
where
    T: GraphLike<'a, K>,
    K: Eq + Hash,
{
    graph_binding: GraphBinding<'a, T, K>,
}

impl<'a, T, K> Resolver<'a, T, K>
where
    T: GraphLike<'a, K> + Debug + 'a,
    K: Eq + Hash + Debug + 'a,
{
    pub fn new(nodes: Vec<&'a T>) -> Result<Resolver<'a, T, K>, Error> {
        let graph_binding = GraphBinding::new(nodes)?;
        Ok(Resolver { graph_binding })
    }

    pub fn resolve(&'a self, keys: Vec<K>) -> Result<Vec<&T>, Error> {
        let nodes = self.graph_binding.find_nodes_by_keys(keys)?;
        self.resolve_nodes(nodes)
    }

    pub fn resolve_all(&'a self) -> Result<Vec<&T>, Error> {
        let nodes = self.graph_binding.get_all_nodes();
        self.resolve_nodes(nodes)
    }

    fn resolve_nodes(&'a self, nodes: Vec<&'a T>) -> Result<Vec<&T>, Error> {
        Ok(self
            .graph_binding
            .topological_sort(nodes)
            .collect::<Vec<&T>>())
    }
}
