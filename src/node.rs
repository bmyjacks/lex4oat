use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A global counter used to generate unique IDs for nodes.
static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Increments the global counter and returns the new value.
///
/// # Returns
///
/// The next unique identifier as a `usize`.
pub fn increment_global_counter() -> usize {
    GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}

#[derive(Clone)]
/// Represents an edge between nodes in a finite automaton.
///
/// The edge connects to a node identified by a unique ID and carries a label.
pub struct Edge {
    // The ID of the node where the edge ends.
    to: usize,
    // The label or symbol associated with the edge.
    path: String,
}

impl Edge {
    /// Creates a new instance of `Edge`.
    ///
    /// # Arguments
    ///
    /// * `to` - The ID of the destination node.
    /// * `name` - The label or symbol for the edge.
    ///
    /// # Returns
    ///
    /// A new `Edge` instance.
    pub fn new(to: usize, name: String) -> Edge {
        Edge { to, path: name }
    }

    /// Retrieves the destination node ID of this edge.
    ///
    /// # Returns
    ///
    /// The unique identifier of the node this edge leads to.
    pub fn get_to(&self) -> usize {
        self.to
    }

    /// Retrieves the label of the edge.
    ///
    /// # Returns
    ///
    /// A string slice representing the edge's symbol.
    pub fn get_sym(&self) -> &str {
        &self.path
    }

    /// Appends a character to the edge's label.
    ///
    /// # Arguments
    ///
    /// * `ch` - The character to be appended.
    pub fn push(&mut self, ch: char) {
        self.path.push(ch);
    }
}

#[derive(Clone)]
/// Represents a node within a finite automaton used for lexical analysis.
///
/// Each node has a unique identifier, a name (which can serve as a token label),
/// outgoing edges, and a flag indicating whether it is a terminal (accepting) state.
pub struct Node {
    /// The name or label of the node.
    name: String,
    /// A list of outgoing edges from the node.
    outgoing_edges: Vec<Edge>,
    /// A unique identifier for the node.
    id: usize,
    /// Indicates whether this node is a terminal (accepting) state.
    terminal: bool,
}

impl Node {
    /// Retrieves the name of the node.
    ///
    /// # Returns
    ///
    /// A string slice representing the node's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Sets a new name for the node.
    ///
    /// # Arguments
    ///
    /// * `name` - The new name to be set.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Retrieves a mutable reference to the node's outgoing edges.
    ///
    /// # Returns
    ///
    /// A mutable vector of `Edge` instances.
    pub fn get_mut_outgoing_edges(&mut self) -> &mut Vec<Edge> {
        &mut self.outgoing_edges
    }

    /// Retrieves a reference to the node's outgoing edges.
    ///
    /// # Returns
    ///
    /// A vector of `Edge` instances.
    pub fn get_outgoing_edges(&self) -> &Vec<Edge> {
        &self.outgoing_edges
    }

    /// Retrieves the unique identifier of the node.
    ///
    /// # Returns
    ///
    /// The node's unique ID as a `usize`.
    pub fn get_id(&self) -> usize {
        self.id
    }

    /// Checks if the node is a terminal (accepting) state.
    ///
    /// # Returns
    ///
    /// `true` if the node is terminal, `false` otherwise.
    pub fn is_terminal(&self) -> bool {
        self.terminal
    }

    /// Sets the terminal (accepting) status of the node.
    ///
    /// # Arguments
    ///
    /// * `terminal` - A boolean indicating the terminal status.
    pub fn set_terminal(&mut self, terminal: bool) {
        self.terminal = terminal;
    }

    /// Creates a new node with a given name and terminal flag.
    ///
    /// A unique identifier is generated using a global counter.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the node.
    /// * `terminal` - Boolean flag to set if the node is terminal.
    ///
    /// # Returns
    ///
    /// A new `Node` instance.
    pub fn new(name: String, terminal: bool) -> Node {
        let id = increment_global_counter();
        Node {
            name,
            outgoing_edges: Vec::new(),
            id,
            terminal,
        }
    }

    /// Adds an outgoing edge from the node.
    ///
    /// # Arguments
    ///
    /// * `to` - The ID of the destination node.
    /// * `name` - The label for the outgoing edge.
    pub fn add_outgoing_edge(&mut self, to: usize, name: String) {
        self.outgoing_edges.push(Edge::new(to, name));
    }

    /// Generates a DOT format representation of the finite automaton starting from this node.
    ///
    /// This method traverses the automaton and outputs its structure in a format that can be
    /// visualized using graph visualization tools.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A reference to a map of node IDs to `Node` instances.
    ///
    /// # Returns
    ///
    /// A `String` containing the DOT representation of the automaton.
    pub fn to_dot(&self, nodes: &HashMap<usize, Node>) -> String {
        let mut dot_string = String::from("digraph FA {\n");
        let mut visited = HashSet::new();
        self.write_dot(&mut dot_string, nodes, &mut visited);
        dot_string.push_str("}\n");
        dot_string
    }

    /// Recursively writes the DOT representation for the node and its descendants.
    ///
    /// This private helper function keeps track of visited nodes to prevent infinite recursion.
    ///
    /// # Arguments
    ///
    /// * `dot_string` - A mutable reference to the DOT format string being constructed.
    /// * `nodes` - A reference to the map of node IDs to `Node` instances.
    /// * `visited` - A mutable set of visited node IDs.
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
                .path
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
