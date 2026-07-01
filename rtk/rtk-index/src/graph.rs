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

        // Map callee name -> all node indices with that name.
        let mut name_to_nodes: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        for (idx, node) in graph.node_weights().enumerate() {
            name_to_nodes
                .entry(node.name.clone())
                .or_default()
                .push(NodeIndex::new(idx));
        }

        // File path per node index, for locality-based callee resolution.
        let node_file: Vec<String> = graph.node_weights().map(|n| n.file_path.clone()).collect();

        // Resolve edges first (immutable reads), then add them (avoids a
        // &graph / &mut graph borrow clash).
        let mut edges: Vec<(NodeIndex, NodeIndex)> = Vec::new();
        for dep in dependencies {
            let Some(&caller_node) = symbol_to_node.get(&dep.caller_id) else {
                continue;
            };
            let Some(cands) = name_to_nodes.get(&dep.callee_name) else {
                continue;
            };
            for target in resolve_callee(caller_node, cands, &node_file) {
                edges.push((caller_node, target));
            }
        }
        for (from, to) in edges {
            graph.add_edge(from, to, ());
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

    /// Trace the downstream call tree from `entry_id`, bounded by `max_depth`
    /// (levels) and `max_nodes` (total). Cycle- and revisit-safe: a callee
    /// already seen on this trace is shown once as a truncated leaf, never
    /// recursed into. Returns `None` if the entry symbol is not indexed.
    pub fn trace_flow(
        &self,
        entry_id: &str,
        max_depth: usize,
        max_nodes: usize,
    ) -> Option<FlowTrace> {
        let start = *self.symbol_to_node.get(entry_id)?;
        let mut visited = HashSet::new();
        let mut stats = TraceStats::default();
        let root = self.build_flow_node(start, 0, max_depth, max_nodes, &mut visited, &mut stats);
        Some(FlowTrace {
            node_count: stats.node_count,
            max_depth_reached: stats.max_depth_reached,
            revisits: stats.revisits,
            capped: stats.capped,
            root,
        })
    }

    fn build_flow_node(
        &self,
        node: NodeIndex,
        depth: usize,
        max_depth: usize,
        max_nodes: usize,
        visited: &mut HashSet<NodeIndex>,
        stats: &mut TraceStats,
    ) -> FlowNode {
        let sym = self
            .graph
            .node_weight(node)
            .cloned()
            .expect("graph node exists");
        stats.node_count += 1;
        stats.max_depth_reached = stats.max_depth_reached.max(depth);
        visited.insert(node);

        // Unique outgoing callee targets, deduped (parallel edges collapse).
        let targets: Vec<NodeIndex> = {
            let mut seen = HashSet::new();
            self.graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .map(|e| e.target())
                .filter(|t| seen.insert(*t))
                .collect()
        };

        let mut children = Vec::new();
        let mut truncated = false;

        if depth >= max_depth {
            truncated = !targets.is_empty();
        } else {
            for t in targets {
                if stats.node_count >= max_nodes {
                    stats.capped = true;
                    truncated = true;
                    break;
                }
                if visited.contains(&t) {
                    stats.revisits += 1;
                    let csym = self
                        .graph
                        .node_weight(t)
                        .cloned()
                        .expect("graph node exists");
                    children.push(FlowNode {
                        name: csym.name,
                        kind: csym.kind,
                        file_path: csym.file_path,
                        line_start: csym.line_start,
                        depth: depth + 1,
                        truncated: true,
                        children: Vec::new(),
                    });
                    continue;
                }
                children.push(self.build_flow_node(
                    t,
                    depth + 1,
                    max_depth,
                    max_nodes,
                    visited,
                    stats,
                ));
            }
        }

        FlowNode {
            name: sym.name,
            kind: sym.kind,
            file_path: sym.file_path,
            line_start: sym.line_start,
            depth,
            truncated,
            children,
        }
    }
}

/// Resolve a call (by name) to the most likely definition(s) using file/module
/// locality, since the index records callees by name only:
/// unique name wins; else prefer a same-file def, then same-directory defs;
/// otherwise fall back to all same-named defs only when the ambiguity is small
/// (<= FALLBACK_LIMIT), else drop the edge (too ambiguous to be meaningful).
fn resolve_callee(
    caller: NodeIndex,
    candidates: &[NodeIndex],
    node_file: &[String],
) -> Vec<NodeIndex> {
    const FALLBACK_LIMIT: usize = 3;
    let cands: Vec<NodeIndex> = candidates
        .iter()
        .copied()
        .filter(|&n| n != caller)
        .collect();
    if cands.len() <= 1 {
        return cands;
    }
    let caller_file = node_file[caller.index()].as_str();

    let same_file: Vec<NodeIndex> = cands
        .iter()
        .copied()
        .filter(|&n| node_file[n.index()] == caller_file)
        .collect();
    if !same_file.is_empty() {
        return same_file;
    }

    let caller_dir = dir_of(caller_file);
    let same_dir: Vec<NodeIndex> = cands
        .iter()
        .copied()
        .filter(|&n| dir_of(&node_file[n.index()]) == caller_dir)
        .collect();
    if !same_dir.is_empty() {
        return same_dir;
    }

    if cands.len() <= FALLBACK_LIMIT {
        cands
    } else {
        Vec::new()
    }
}

fn dir_of(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

#[derive(Default)]
struct TraceStats {
    node_count: usize,
    max_depth_reached: usize,
    revisits: usize,
    capped: bool,
}

/// A node in a downstream execution-flow tree.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FlowNode {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line_start: usize,
    pub depth: usize,
    /// True when recursion stopped here (depth/node cap or an already-seen node).
    pub truncated: bool,
    pub children: Vec<FlowNode>,
}

/// A traced execution flow plus summary stats.
#[derive(Debug, serde::Serialize)]
pub struct FlowTrace {
    pub node_count: usize,
    pub max_depth_reached: usize,
    pub revisits: usize,
    pub capped: bool,
    pub root: FlowNode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{DbDependency, DbSymbol};

    fn sym(id: &str, name: &str) -> DbSymbol {
        DbSymbol {
            id: id.to_string(),
            name: name.to_string(),
            kind: "Function".to_string(),
            file_path: format!("{}.rs", name),
            line_start: 1,
            line_end: 2,
        }
    }
    fn dep(caller: &str, callee: &str) -> DbDependency {
        DbDependency {
            caller_id: caller.to_string(),
            callee_name: callee.to_string(),
            callee_file_path: None,
            dependency_kind: "call".to_string(),
        }
    }

    #[test]
    fn test_trace_flow_depth_cap_and_cycle() {
        // a -> b -> c -> a (cycle)
        let syms = vec![sym("1", "a"), sym("2", "b"), sym("3", "c")];
        let deps = vec![dep("1", "b"), dep("2", "c"), dep("3", "a")];
        let g = ImpactGraph::build(syms, deps);

        // Depth 1: a -> b only (b's children not expanded).
        let t1 = g.trace_flow("1", 1, 100).unwrap();
        assert_eq!(t1.root.name, "a");
        assert_eq!(t1.root.children.len(), 1);
        assert_eq!(t1.root.children[0].name, "b");
        assert!(t1.root.children[0].children.is_empty());

        // Deep trace terminates on the cycle without looping forever.
        let t = g.trace_flow("1", 10, 100).unwrap();
        assert!(t.revisits >= 1, "cycle back to 'a' should be a revisit");
        assert!(t.node_count <= 6);
    }

    #[test]
    fn test_build_prefers_same_file_callee() {
        // Two defs named "helper": one in a.rs (same file as caller), one in z.rs.
        // The call from `caller` in a.rs must resolve only to the a.rs helper.
        let mut a_helper = sym("1", "helper");
        a_helper.file_path = "a.rs".into();
        let mut z_helper = sym("2", "helper");
        z_helper.file_path = "z.rs".into();
        let mut caller = sym("3", "caller");
        caller.file_path = "a.rs".into();

        let g = ImpactGraph::build(vec![a_helper, z_helper, caller], vec![dep("3", "helper")]);
        let downstream = g.resolve_downstream("3");
        assert_eq!(downstream.len(), 1);
        assert_eq!(downstream[0].id, "1"); // the same-file helper, not z.rs
    }

    #[test]
    fn test_build_drops_highly_ambiguous_callee() {
        // "get" defined in 4 unrelated dirs, caller elsewhere → no local/dir match
        // and >3 candidates → edge dropped (no false connections).
        let mk = |id: &str, dir: &str| {
            let mut s = sym(id, "get");
            s.file_path = format!("{dir}/f.rs");
            s
        };
        let mut caller = sym("9", "caller");
        caller.file_path = "app/x.rs".into();
        let g = ImpactGraph::build(
            vec![
                mk("1", "t1"),
                mk("2", "t2"),
                mk("3", "t3"),
                mk("4", "t4"),
                caller,
            ],
            vec![dep("9", "get")],
        );
        assert_eq!(g.resolve_downstream("9").len(), 0);
    }

    #[test]
    fn test_trace_flow_node_cap() {
        // a -> b, a -> c, a -> d ; cap at 2 nodes
        let syms = vec![sym("1", "a"), sym("2", "b"), sym("3", "c"), sym("4", "d")];
        let deps = vec![dep("1", "b"), dep("1", "c"), dep("1", "d")];
        let g = ImpactGraph::build(syms, deps);
        let t = g.trace_flow("1", 6, 2).unwrap();
        assert!(t.capped);
        assert!(t.node_count <= 2);
    }
}
