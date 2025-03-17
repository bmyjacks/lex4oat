//! Module for constructing a deterministic finite automaton (DFA) from a nondeterministic finite automaton (NFA).
//! It provides functionalities for creating a DFA, computing epsilon closures, moving on symbols, and lexing input strings.

use crate::nfa::Nfa;
use crate::node::Node;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::rc::Rc;

/// Represents a deterministic finite automaton (DFA).
pub struct Dfa {
    /// Shared reference to the underlying NFA.
    nfa: Rc<RefCell<Nfa>>,
    /// Map of DFA node IDs to their corresponding `Node` structures.
    nodes: HashMap<usize, Node>,
    /// The root node ID of the DFA.
    root_id: usize,
}

impl Dfa {
    /// Creates a new DFA by initializing an empty NFA backbone and a root node.
    ///
    /// # Returns
    /// A new instance of `Dfa`.
    pub fn new() -> Dfa {
        let nfa = Rc::new(RefCell::new(Nfa::new()));
        let root = Node::new("DFA".to_string(), false);
        let root_id = root.get_id();
        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);
        Dfa {
            nfa,
            nodes,
            root_id,
        }
    }

    /// Sets the internal NFA for this DFA.
    ///
    /// # Arguments
    ///
    /// * `nfa` - A shared pointer with interior mutability to the NFA.
    pub fn set_nfa(&mut self, nfa: Rc<RefCell<Nfa>>) {
        self.nfa = nfa;
    }

    /// Constructs the DFA using subset construction starting from the epsilon closure of the NFA's root.
    /// It populates the set of DFA nodes and writes the resulting DFA graph in DOT format to a file.
    pub fn construct_dfa(&mut self) {
        let mut dfa_states: HashMap<BTreeSet<usize>, usize> = HashMap::new();
        let mut unmarked: VecDeque<BTreeSet<usize>> = VecDeque::new();

        // Compute the epsilon closure for the NFA's root.
        let nfa_root_id = self.nfa.borrow().get_root_id();
        let start_set: BTreeSet<usize> = [nfa_root_id].iter().cloned().collect();
        let start_closure = self.epsilon_closure(&start_set);
        let start_dfa_id = self.create_dfa_state(&start_closure);
        self.root_id = start_dfa_id;
        dfa_states.insert(start_closure.clone(), start_dfa_id);
        unmarked.push_back(start_closure);

        // Process unmarked states until no more states remain.
        while let Some(current_set) = unmarked.pop_front() {
            let current_dfa_id = dfa_states[&current_set];

            // Extract all available transition symbols from the current set.
            let symbols = self.extract_symbols(&current_set, self.nfa.borrow().get_nodes());
            for ch in symbols {
                // Determine the set of NFA states reachable by symbol ch including epsilon moves.
                let move_set = self.move_nfa(&current_set, &ch);
                let closure = self.epsilon_closure(&move_set);
                if closure.is_empty() {
                    continue;
                }

                // Check if the DFA state corresponding to the closure already exists.
                let next_dfa_id = if let Some(&id) = dfa_states.get(&closure) {
                    id
                } else {
                    let new_id = self.create_dfa_state(&closure);
                    dfa_states.insert(closure.clone(), new_id);
                    unmarked.push_back(closure.clone());
                    new_id
                };

                // Add or update the outgoing edge for the current DFA node.
                let current_dfa_node = self.nodes.get_mut(&current_dfa_id).unwrap();
                if let Some(edge) = current_dfa_node
                    .get_mut_outgoing_edges()
                    .iter_mut()
                    .find(|edge| edge.get_to() == next_dfa_id)
                {
                    edge.push(ch);
                } else {
                    current_dfa_node.add_outgoing_edge(next_dfa_id, ch.to_string());
                }
            }
        }

        // Write the DFA in DOT format to a file.
        let dot_string = self.nodes.get(&self.root_id).unwrap().to_dot(&self.nodes);
        std::fs::write("dfa.dot", dot_string).expect("Failed to write DFA to file");
    }

    /// Computes the epsilon closure of a given set of NFA state IDs.
    ///
    /// # Arguments
    ///
    /// * `state_set` - A set of NFA state IDs.
    ///
    /// # Returns
    /// A set containing all state IDs reachable from `state_set` using epsilon transitions.
    fn epsilon_closure(&self, state_set: &BTreeSet<usize>) -> BTreeSet<usize> {
        let mut closure = state_set.clone();
        let mut stack: Vec<usize> = state_set.iter().cloned().collect();

        while let Some(state_id) = stack.pop() {
            if let Some(nfa_node) = self.nfa.borrow().get_nodes().get(&state_id) {
                for edge in nfa_node.get_outgoing_edges().iter() {
                    let to = edge.get_to();
                    let name = edge.get_sym().to_string();
                    if name == "<λ>" && !closure.contains(&to) {
                        closure.insert(to);
                        stack.push(to);
                    }
                }
            }
        }

        closure
    }

    /// Computes the set of NFA states reachable from a given state set by consuming the specified character.
    ///
    /// # Arguments
    ///
    /// * `state_set` - A set of NFA state IDs.
    /// * `ch` - The transition character.
    ///
    /// # Returns
    /// A set containing all state IDs reached over transitions labeled with `ch`.
    fn move_nfa(&mut self, state_set: &BTreeSet<usize>, ch: &char) -> BTreeSet<usize> {
        let mut result = BTreeSet::new();
        for state_id in state_set {
            if let Some(nfa_node) = self.nfa.borrow().get_nodes().get(state_id) {
                for edge in nfa_node.get_outgoing_edges().iter() {
                    let to = edge.get_to();
                    let sym = edge.get_sym();

                    // Check for transition on the provided character.
                    if sym.contains(*ch) && !result.contains(&edge.get_to()) {
                        result.insert(to);
                    }
                }
            }
        }
        result
    }

    /// Extracts all non-epsilon symbols available from the transitions of NFA states.
    ///
    /// # Arguments
    ///
    /// * `state_set` - A set of NFA state IDs.
    /// * `_nodes` - A map of NFA nodes.
    ///
    /// # Returns
    /// A set of characters representing all transition symbols excluding epsilon.
    fn extract_symbols(
        &self,
        state_set: &BTreeSet<usize>,
        _nodes: &HashMap<usize, Node>,
    ) -> BTreeSet<char> {
        let mut result = BTreeSet::new();
        for state_id in state_set {
            if let Some(nfa_node) = self.nfa.borrow().get_nodes().get(state_id) {
                for edge in nfa_node.get_outgoing_edges().iter() {
                    let sym = edge.get_sym().to_string();
                    if sym != "<λ>" {
                        for ch in sym.chars() {
                            result.insert(ch);
                        }
                    }
                }
            }
        }
        result
    }

    /// Creates a new DFA state based on a set of NFA states.
    ///
    /// This method determines whether the new DFA state is accepting based on the underlying NFA nodes.
    ///
    /// # Arguments
    ///
    /// * `state_set` - A set of NFA state IDs.
    ///
    /// # Returns
    /// The newly created DFA state's identifier.
    fn create_dfa_state(&mut self, state_set: &BTreeSet<usize>) -> usize {
        // Collect names of terminal nodes.
        let terminal_names: Vec<String> = state_set
            .iter()
            .filter_map(|id| {
                self.nfa.borrow().get_nodes().get(id).and_then(|node| {
                    if node.is_terminal() {
                        Some(node.get_name().to_string())
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Determine if the DFA state should be a terminal state.
        let is_terminal = state_set.iter().any(|&id| {
            self.nfa
                .borrow()
                .get_nodes()
                .get(&id)
                .map_or(false, |node| node.is_terminal())
        });

        let name = if is_terminal {
            terminal_names[0].clone()
        } else {
            "<>".to_string()
        };

        let new_node = Node::new(name, is_terminal);
        let new_node_id = new_node.get_id();
        self.nodes.insert(new_node_id, new_node);
        new_node_id
    }

    /// Lexes the input string using the constructed DFA.
    ///
    /// Iterates through the input characters, traversing the DFA transitions until a valid token is found.
    /// Returns a vector containing tuples of state names and token lexemes.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string to be lexed.
    ///
    /// # Returns
    /// A vector of tuples where each tuple represents (state name, token).
    pub fn lex(&mut self, input: &str) -> Vec<(String, String)> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut index = 0;

        // Iterate over the input characters.
        while index < chars.len() {
            let mut current_state_id = self.root_id;
            let mut last_accept_index: Option<usize> = None;
            let mut last_accept_state_name = String::new();
            let mut j = index;

            // Traverse the DFA transitions for as long as possible.
            while j < chars.len() {
                let current_node = self.nodes.get(&current_state_id).unwrap();
                let mut found = false;

                // Check for transitions matching the current character.
                for edge in current_node.get_outgoing_edges().iter() {
                    if edge.get_sym().contains(&chars[j].to_string()) {
                        found = true;
                        current_state_id = edge.get_to();
                        break;
                    }
                }

                if !found {
                    break;
                }

                // Record accepted state if terminal.
                if self.nodes.get(&current_state_id).unwrap().is_terminal() {
                    last_accept_index = Some(j + 1);
                    last_accept_state_name = self
                        .nodes
                        .get(&current_state_id)
                        .unwrap()
                        .get_name()
                        .to_string();
                }
                j += 1;
            }

            // If an accepted state was found, extract the token.
            if let Some(end_index) = last_accept_index {
                let token: String = chars[index..end_index].iter().collect();
                let token = token.trim().to_string();
                if last_accept_state_name != ";" {
                    tokens.push((last_accept_state_name, token));
                }
                index = end_index;
            } else {
                index += 1;
            }
        }

        tokens
    }
}
