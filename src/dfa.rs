// rust
use crate::nfa::Nfa;
use crate::node::Node;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::rc::Rc;

pub struct Dfa {
    nfa: Rc<RefCell<Nfa>>,       // NFA backbone
    nodes: HashMap<usize, Node>, // DFA's node set
    root_id: usize,              // DFA's root_id
}

impl Dfa {
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

    // Set the nfa using a shared pointer with interior mutability.
    pub fn set_nfa(&mut self, nfa: Rc<RefCell<Nfa>>) {
        self.nfa = nfa;
    }

    pub fn construct_dfa(&mut self) {
        let mut dfa_states: HashMap<BTreeSet<usize>, usize> = HashMap::new();
        let mut unmarked: VecDeque<BTreeSet<usize>> = VecDeque::new();

        // epsilon closure for NFA's root
        let nfa_root_id = self.nfa.borrow().get_root_id();
        let start_set: BTreeSet<usize> = [nfa_root_id].iter().cloned().collect();
        let start_closure = self.epsilon_closure(&start_set);
        let start_dfa_id = self.create_dfa_state(&start_closure);
        self.root_id = start_dfa_id;
        dfa_states.insert(start_closure.clone(), start_dfa_id);
        unmarked.push_back(start_closure);

        while let Some(current_set) = unmarked.pop_front() {
            let current_dfa_id = dfa_states[&current_set];

            // Gather all avail trans symbols from the current set.
            let symbols = self.extract_symbols(&current_set, self.nfa.borrow().get_nodes());
            for ch in symbols {
                // Gather all states reached on sym and then epsilon
                let move_set = self.move_nfa(&current_set, &ch);
                let closure = self.epsilon_closure(&move_set);
                if closure.is_empty() {
                    continue;
                }

                // Check if this DFA state already exists
                let next_dfa_id = if let Some(&id) = dfa_states.get(&closure) {
                    id
                } else {
                    let new_id = self.create_dfa_state(&closure);
                    dfa_states.insert(closure.clone(), new_id);
                    unmarked.push_back(closure.clone());
                    new_id
                };

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

        let dot_string = self.nodes.get(&self.root_id).unwrap().to_dot(&self.nodes);
        std::fs::write("dfa.dot", dot_string).expect("Failed to write DFA to file");
    }

    // Compute the epsilon closure of a set of NFA states.
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

    // Move from each state in state_set using the provided char, which is now a string slice.
    fn move_nfa(&mut self, state_set: &BTreeSet<usize>, ch: &char) -> BTreeSet<usize> {
        let mut result = BTreeSet::new();
        for state_id in state_set {
            if let Some(nfa_node) = self.nfa.borrow().get_nodes().get(state_id) {
                for edge in nfa_node.get_outgoing_edges().iter() {
                    let to = edge.get_to();
                    let sym = edge.get_sym();

                    // Check if arrive to using ch
                    if sym.contains(*ch) && !result.contains(&edge.get_to()) {
                        result.insert(to);
                    }
                }
            }
        }
        result
    }

    // Extract all non-lambda symbols available from transitions of nodes in state_set.
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

        // Determine DFA state name.

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

    // Updated lex method that works with string symbols.
    pub fn lex(&mut self, input: &str) -> Vec<(String, String)> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut index = 0;

        while index < chars.len() {
            let mut current_state_id = self.root_id;
            let mut last_accept_index: Option<usize> = None;
            let mut last_accept_state_name = String::new();
            let mut j = index;

            while j < chars.len() {
                let current_node = self.nodes.get(&current_state_id).unwrap();
                let mut found = false;

                for edge in current_node.get_outgoing_edges().iter() {
                    // Compare using contains. The edge's string label may consist of multiple characters.
                    if edge.get_sym().contains(&chars[j].to_string()) {
                        found = true;
                        current_state_id = edge.get_to();
                        break;
                    }
                }

                if !found {
                    break;
                }

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
