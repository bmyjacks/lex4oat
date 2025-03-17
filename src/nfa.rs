// Use Node for NFA node representation.
use crate::node::Node;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a nondeterministic finite automaton (NFA) used for lexical analysis.
pub struct Nfa {
    /// A list of keyword definitions where each tuple contains the regex and its token name.
    keywords: Vec<(String, String)>,
    /// A map of node IDs to their corresponding Node structures.
    nodes: HashMap<usize, Node>,
    /// The ID of the root node of the NFA.
    root_id: usize,
}

impl Nfa {
    /// Returns a reference to the NFA nodes.
    pub fn get_nodes(&self) -> &HashMap<usize, Node> {
        &self.nodes
    }

    /// Returns the ID of the root node.
    pub fn get_root_id(&self) -> usize {
        self.root_id
    }

    /// Creates a new NFA with an initial root node.
    pub fn new() -> Nfa {
        let root = Node::new("NFA".to_string(), false);
        let root_id = root.get_id();
        let mut nodes = HashMap::new();
        nodes.insert(root_id, root);
        Nfa {
            keywords: Vec::new(),
            nodes,
            root_id,
        }
    }

    /// Reads keywords from a file and adds them to the NFA.
    ///
    /// The file is expected to contain lines where each keyword is paired with its token name.
    /// Lines starting with "%%" or empty lines are ignored.
    ///
    /// # Arguments
    ///
    /// * `file` - A reference to the file path containing the keywords.
    pub fn add_keywords_from_file(&mut self, file: &PathBuf) {
        let input = std::fs::read_to_string(file).expect("Failed to read input file");
        let lines = input.lines();

        for line in lines {
            if line.starts_with("%%") || line.is_empty() {
                continue;
            }

            let parts = line.split_whitespace().collect::<Vec<&str>>();
            let name = parts.last().unwrap().trim_matches('"').to_string();
            let keyword = parts[..parts.len() - 1].join(" ");
            self.keywords.push((keyword, name));
        }
    }

    /// Parses a regex set (character class) and connects it to an existing node.
    ///
    /// This method handles both normal and negated sets.
    ///
    /// # Arguments
    ///
    /// * `regex` - The string representation of the character set.
    /// * `name` - The token name associated with this set.
    /// * `start_node_id` - The ID of the starting node.
    ///
    /// # Returns
    ///
    /// The new node ID created for the character set.
    pub fn parse_regex_set(&mut self, regex: &str, name: &str, start_node_id: usize) -> usize {
        let mut chars = regex.chars().peekable();

        let new_node = Node::new(name.to_owned(), false);
        let new_node_id = new_node.get_id();

        // Check if the set is negated with a '^' at the start.
        let mut is_negated = false;
        if let Some(&first) = chars.peek() {
            if first == '^' {
                is_negated = true;
                chars.next(); // consume '^'
            }
        }

        let mut set_chars: Vec<char> = Vec::new();
        let mut prev_char: Option<char> = None;

        while let Some(c) = chars.next() {
            if c == '\\' {
                // Process escaped characters.
                let next = chars.next().unwrap();
                match next {
                    's' => {
                        set_chars.push(' ');
                        set_chars.push('\t');
                        set_chars.push('\n');
                        set_chars.push('\r');
                    }
                    _ => {
                        set_chars.push(next);
                    }
                }
                prev_char = Some(next);
            } else if c == '-' {
                // Process range.
                if let Some(start) = prev_char {
                    let end_char = chars.next().unwrap();
                    for ch in ((start as u8 + 1) as char)..=end_char {
                        set_chars.push(ch);
                    }
                    prev_char = Some(end_char);
                }
            } else {
                set_chars.push(c);
                prev_char = Some(c);
            }
        }

        let mut edge_name = String::new();

        if is_negated {
            // For negated sets, add transitions for all ASCII characters not in set_chars.
            for code in 32u8..=126u8 {
                let ch = code as char;
                if !set_chars.contains(&ch) {
                    edge_name.push(ch);
                }
            }
        } else {
            // For normal sets, add transitions for each character in the set.
            for ch in set_chars {
                edge_name.push(ch);
            }
        }

        self.nodes
            .get_mut(&start_node_id)
            .unwrap()
            .add_outgoing_edge(new_node_id, edge_name);
        self.nodes.insert(new_node_id, new_node);
        new_node_id
    }

    /// Parses a regex group (parenthesized expression) and connects it to an existing node.
    ///
    /// A lambda transition is added between the start node and group start node.
    ///
    /// # Arguments
    ///
    /// * `regex` - The inner group regex.
    /// * `name` - The token name associated with this group.
    /// * `start_node_id` - The starting node ID before the group.
    ///
    /// # Returns
    ///
    /// The final node ID after parsing the group.
    pub fn parse_regex_group(&mut self, regex: &str, name: &str, start_node_id: usize) -> usize {
        // Create a group start node.
        let group_start = Node::new("(".to_string(), false);
        let group_start_id = group_start.get_id();
        self.nodes.insert(group_start_id, group_start);

        // Connect the previous node (start_node_id) to the group start node using a lambda transition.
        self.nodes
            .get_mut(&start_node_id)
            .unwrap()
            .add_outgoing_edge(group_start_id, "<λ>".to_string());

        // Recursively parse the inner group expression starting at group_start.
        let group_end_id = self.parse_regex(regex, name, group_start_id, false);

        // Return the final node of the group.
        group_end_id
    }

    /// Parses a regex pattern and constructs corresponding NFA nodes and transitions.
    ///
    /// This method supports alternation, escaped characters, character classes, groups,
    /// and repetition operators (*, +, ?).
    ///
    /// # Arguments
    ///
    /// * `regex` - The regex pattern to parse.
    /// * `name` - The token name associated with the pattern.
    /// * `start_node_id` - The starting node ID for the regex.
    /// * `mark_ending` - A boolean indicating whether the ending node should be marked as terminal.
    ///
    /// # Returns
    ///
    /// The ID of the ending node for the parsed regex.
    pub fn parse_regex(
        &mut self,
        regex: &str,
        name: &str,
        start_node_id: usize,
        mark_ending: bool,
    ) -> usize {
        let mut chars = regex.chars().peekable();

        // Save the branch start to be used for all alternates.
        let branch_start = *(&start_node_id);
        let mut alternatives: Vec<usize> = Vec::new();

        // Our stack starts with the branch start.
        let mut stack: Vec<usize> = Vec::new();
        stack.push(branch_start);

        let mut end_node_id = start_node_id;

        while let Some(c) = chars.next() {
            if c == '|' {
                // End current alternative branch.
                let current_alt = *stack.last().unwrap();
                alternatives.push(current_alt);
                // Reset the branch by replacing current branch with branch_start.
                stack.pop();
                stack.push(branch_start);
                continue;
            } else if c == '\\' {
                let next = *chars.peek().unwrap();
                match next {
                    's' => {
                        let new_node = Node::new(name.to_string(), false);
                        let new_node_id = new_node.get_id();
                        self.nodes.insert(new_node_id, new_node);
                        for ws in [" ", "\t", "\n", "\r"] {
                            self.nodes
                                .get_mut(stack.last().unwrap())
                                .unwrap()
                                .add_outgoing_edge(new_node_id, ws.to_string());
                        }
                        stack.push(new_node_id);
                        chars.next();
                    }
                    _ => {
                        chars.next();
                        let new_node = Node::new(next.to_string(), false);
                        let new_node_id = new_node.get_id();
                        self.nodes.insert(new_node_id, new_node);
                        self.nodes
                            .get_mut(stack.last().unwrap())
                            .unwrap()
                            .add_outgoing_edge(new_node_id, next.to_string());
                        stack.push(new_node_id);
                        if chars.peek().is_none() && mark_ending {
                            self.nodes
                                .get_mut(stack.last().unwrap())
                                .unwrap()
                                .set_terminal(true);
                            self.nodes
                                .get_mut(stack.last().unwrap())
                                .unwrap()
                                .set_name(name.to_string());
                            end_node_id = new_node_id;
                        }
                    }
                }
            } else if c == '[' {
                let mut char_set = String::new();
                while let Some(c) = chars.next() {
                    if c == ']' {
                        break;
                    }
                    char_set.push(c);
                }
                let current_node_id = *stack.last().unwrap();
                let result_node_id = self.parse_regex_set(char_set.as_str(), name, current_node_id);
                stack.push(result_node_id);
                if chars.peek().is_none() && mark_ending {
                    self.nodes
                        .get_mut(&result_node_id)
                        .unwrap()
                        .set_terminal(true);
                }
            } else if c == '(' {
                let mut group_expr = String::new();
                while let Some(c) = chars.next() {
                    if c == ')' {
                        break;
                    }
                    group_expr.push(c);
                }
                let current_node_id = *stack.last().unwrap();
                let result_node_id =
                    self.parse_regex_group(group_expr.as_str(), name, current_node_id);
                stack.push(result_node_id);
                if chars.peek().is_none() && mark_ending {
                    self.nodes
                        .get_mut(&result_node_id)
                        .unwrap()
                        .set_terminal(true);
                }
            } else if c == '*' {
                let repeat_node_id = stack.pop().unwrap();
                let prev_node_id = *stack.last().unwrap();
                let merge_node = Node::new(name.to_owned(), false);
                let merge_node_id = merge_node.get_id();
                self.nodes.insert(merge_node_id, merge_node);
                stack.push(merge_node_id);
                // Skip the repeated pattern.
                self.nodes
                    .get_mut(&prev_node_id)
                    .unwrap()
                    .add_outgoing_edge(merge_node_id, "<λ>".to_string());
                // Finalize one occurrence.
                self.nodes
                    .get_mut(&repeat_node_id)
                    .unwrap()
                    .add_outgoing_edge(merge_node_id, "<λ>".to_string());
                // Allow repetition.
                self.nodes
                    .get_mut(&merge_node_id)
                    .unwrap()
                    .add_outgoing_edge(prev_node_id, "<λ>".to_string());
                if chars.peek().is_none() && mark_ending {
                    self.nodes
                        .get_mut(&merge_node_id)
                        .unwrap()
                        .set_terminal(true);
                }
            } else if c == '+' {
                let repeat_node_id = stack.pop().unwrap();
                let prev_node_id = *stack.last().unwrap();
                stack.push(repeat_node_id);
                self.nodes
                    .get_mut(&repeat_node_id)
                    .unwrap()
                    .add_outgoing_edge(prev_node_id, "<λ>".to_string());
                if chars.peek().is_none() && mark_ending {
                    self.nodes
                        .get_mut(&repeat_node_id)
                        .unwrap()
                        .set_terminal(true);
                }
            } else if c == '?' {
                let optional_id = stack.pop().unwrap();
                let prev_id = stack.last().unwrap();
                let merge_node = Node::new(c.to_string(), false);
                let merge_node_id = merge_node.get_id();
                self.nodes
                    .get_mut(prev_id)
                    .unwrap()
                    .add_outgoing_edge(merge_node_id, "<λ>".to_string());
                self.nodes
                    .get_mut(&optional_id)
                    .unwrap()
                    .add_outgoing_edge(merge_node_id, "<λ>".to_string());
                self.nodes.insert(merge_node_id, merge_node);
                stack.push(merge_node_id);
            } else {
                let new_node = Node::new(c.to_string(), false);
                let new_node_id = new_node.get_id();
                self.nodes.insert(new_node_id, new_node);
                self.nodes
                    .get_mut(stack.last().unwrap())
                    .unwrap()
                    .add_outgoing_edge(new_node_id, c.to_string());
                stack.push(new_node_id);
                if chars.peek().is_none() && mark_ending {
                    self.nodes.get_mut(&new_node_id).unwrap().set_terminal(true);
                    self.nodes
                        .get_mut(&new_node_id)
                        .unwrap()
                        .set_name(name.to_string());
                    end_node_id = new_node_id;
                }
            }
        }

        // If at least one alternate was detected, merge all branches.
        if !alternatives.is_empty() {
            // Add the current branch endpoint to alternatives.
            alternatives.push(*stack.last().unwrap());
            // Create a merge node.
            let merge_node = Node::new(name.to_string(), false);
            let merge_node_id = merge_node.get_id();
            self.nodes.insert(merge_node_id, merge_node);
            // Connect every alternative branch to the merge node.
            for alt_end in alternatives {
                self.nodes
                    .get_mut(&alt_end)
                    .unwrap()
                    .add_outgoing_edge(merge_node_id, "<λ>".to_string());
            }
            end_node_id = merge_node_id;
        }

        end_node_id
    }

    /// Constructs the NFA by parsing all keywords.
    ///
    /// Each keyword is processed into an NFA fragment and then linked together,
    /// with the final NFA being output in DOT format.
    pub fn construct(&mut self) {
        let keywords = self.keywords.clone();

        for (keyword, name) in &keywords {
            if name == ";" {
                continue;
            }

            let _ = self.parse_regex(keyword, name, self.root_id, true);
        }

        let dot_string = self.nodes.get(&self.root_id).unwrap().to_dot(&self.nodes);
        std::fs::write("nfa.dot", dot_string).expect("Failed to write NFA to file");
    }
}
