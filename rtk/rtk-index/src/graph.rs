use crate::db::{DbDependency, DbSymbol};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

pub struct ImpactGraph {
    graph: DiGraph<DbSymbol, ()>,
    symbol_to_node: HashMap<String, NodeIndex>,
}

impl ImpactGraph {
    pub fn build(symbols: Vec<DbSymbol>, dependencies: Vec<DbDependency>) -> Self {
        let mut graph = DiGraph::new();
        let mut symbol_to_node = HashMap::new();

        // Add all nodes
        for sym in symbols {
            let id = sym.id.clone();
            let idx = graph.add_node(sym);
            symbol_to_node.insert(id, idx);
        }

        // Map callee name to all node indices with that name for easy lookup
        let mut name_to_nodes: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        for (idx, node) in graph.node_weights().enumerate() {
            let n_idx = NodeIndex::new(idx);
            name_to_nodes
                .entry(node.name.clone())
                .or_default()
                .push(n_idx);
        }

        // Add edges
        for dep in dependencies {
            if let Some(&caller_node) = symbol_to_node.get(&dep.caller_id) {
                // Find potential destination node indices matching callee_name
                if let Some(dest_nodes) = name_to_nodes.get(&dep.callee_name) {
                    for &dest_node in dest_nodes {
                        // Don't add self-loops
                        if caller_node != dest_node {
                            graph.add_edge(caller_node, dest_node, ());
                        }
                    }
                }
            }
        }

        Self {
            graph,
            symbol_to_node,
        }
    }

    // Upstream impact analysis (find all callers that depend on target)
    pub fn resolve_upstream(&self, target_id: &str) -> Vec<DbSymbol> {
        let mut affected = Vec::new();
        let mut visited = HashSet::new();

        let start_node = match self.symbol_to_node.get(target_id) {
            Some(&n) => n,
            None => return affected,
        };

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start_node);
        visited.insert(start_node);

        // We traverse backwards (using incoming edges)
        while let Some(current) = queue.pop_front() {
            // Find all incoming edges (callers)
            let incoming = self
                .graph
                .edges_directed(current, petgraph::Direction::Incoming);
            for edge in incoming {
                let caller_node = edge.source();
                if visited.insert(caller_node) {
                    queue.push_back(caller_node);
                    if let Some(sym) = self.graph.node_weight(caller_node) {
                        affected.push(sym.clone());
                    }
                }
            }
        }

        affected
    }

    // Downstream dependency analysis (find all functions that target depends on)
    pub fn resolve_downstream(&self, target_id: &str) -> Vec<DbSymbol> {
        let mut deps = Vec::new();
        let mut visited = HashSet::new();

        let start_node = match self.symbol_to_node.get(target_id) {
            Some(&n) => n,
            None => return deps,
        };

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start_node);
        visited.insert(start_node);

        while let Some(current) = queue.pop_front() {
            let outgoing = self
                .graph
                .edges_directed(current, petgraph::Direction::Outgoing);
            for edge in outgoing {
                let callee_node = edge.target();
                if visited.insert(callee_node) {
                    queue.push_back(callee_node);
                    if let Some(sym) = self.graph.node_weight(callee_node) {
                        deps.push(sym.clone());
                    }
                }
            }
        }

        deps
    }
}
