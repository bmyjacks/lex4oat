// File: src/node.rs
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn increment_global_counter() -> usize {
    GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}

pub fn get_global_counter() -> usize {
    GLOBAL_COUNTER.load(Ordering::SeqCst)
}

pub fn reset_global_counter() {
    GLOBAL_COUNTER.store(0, Ordering::SeqCst);
}

#[derive(Clone)]
pub struct Edge {
    // The ID of the node where the edge starts
    from: usize,
    // The ID of the node where the edge ends
    to: usize,
    // The name of the edge
    name: String,
}

impl Edge {
    pub fn new(from: usize, to: usize, name: String) -> Edge {
        Edge { from, to, name }
    }

    pub fn get_from(&self) -> usize {
        self.from
    }

    pub fn get_to(&self) -> usize {
        self.to
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Clone)]
pub struct Node {
    name: String,
    outgoing_edges: Vec<Edge>,
    id: usize,
    terminal: bool,
}

impl Node {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_mut_outgoing_edges(&mut self) -> &mut Vec<Edge> {
        &mut self.outgoing_edges
    }

    pub fn get_outgoing_edges(&self) -> &Vec<Edge> {
        &self.outgoing_edges
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    pub fn set_terminal(&mut self, terminal: bool) {
        self.terminal = terminal;
    }

    pub fn new(name: String, terminal: bool) -> Node {
        let id = increment_global_counter();
        Node {
            name,
            outgoing_edges: Vec::new(),
            id,
            terminal,
        }
    }

    pub fn add_outgoing_edge(&mut self, to: usize, name: String) {
        self.outgoing_edges.push(Edge::new(self.id, to, name));
    }

    // Revised to_dot function that uses a visited set to prevent infinite loops.
    pub fn to_dot(&self, nodes: &HashMap<usize, Node>) -> String {
        let mut dot_string = String::from("digraph NFA {\n");
        let mut visited = HashSet::new();
        self.write_dot(&mut dot_string, nodes, &mut visited);
        dot_string.push_str("}\n");
        dot_string
    }

    // Revised write_dot function that tracks visited nodes.
    fn write_dot(
        &self,
        dot_string: &mut String,
        nodes: &HashMap<usize, Node>,
        visited: &mut HashSet<usize>,
    ) {
        if visited.contains(&self.id) {
            return;
        }
        visited.insert(self.id);
        for edge in &self.outgoing_edges {
            let to = nodes.get(&edge.to).unwrap();
            let escaped_label = edge
                .name
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\t", "\\\\t")
                .replace("\n", "\\\\n")
                .replace("\r", "\\\\r");
            dot_string.push_str(&format!(
                "    {} -> {} [label=\"{}\"];\n",
                self.id, to.id, escaped_label
            ));
            to.write_dot(dot_string, nodes, visited);
        }
        if self.terminal {
            let escaped_name = self
                .name
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\t", "\\\\t")
                .replace("\n", "\\\\n")
                .replace("\r", "\\\\r");
            dot_string.push_str(&format!(
                "    {} [shape=doublecircle, label=\"{}\"];\n",
                self.id, escaped_name
            ));
        }
    }
}
