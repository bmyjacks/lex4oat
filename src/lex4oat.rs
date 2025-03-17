use crate::dfa::Dfa;
use crate::nfa::Nfa;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

/// A lexer for the Oat language that utilizes both NFA and DFA to perform lexical analysis.
pub struct Lex4Oat {
    /// The input source code to be lexed.
    input: String,
    /// Reference counted, mutable reference to the NFA used for constructing token rules.
    nfa: Rc<RefCell<Nfa>>,
    /// Reference counted, mutable reference to the DFA used for lexing.
    dfa: Rc<RefCell<Dfa>>,
}

impl Lex4Oat {
    /// Creates a new instance of `Lex4Oat` with the provided input string.
    ///
    /// # Arguments
    ///
    /// * `input` - A string containing the source code.
    ///
    /// # Returns
    ///
    /// An instance of `Lex4Oat`.
    pub fn new(input: String) -> Lex4Oat {
        let nfa = Rc::new(RefCell::new(Nfa::new()));
        let dfa = Rc::new(RefCell::new(Dfa::new()));
        Lex4Oat { input, nfa, dfa }
    }

    /// Constructs the NFA by adding keywords from a file and building the overall automaton.
    ///
    /// The keywords are read from the file located at `src/oat.l`.
    pub fn construct_nfa(&mut self) {
        self.nfa
            .borrow_mut()
            .add_keywords_from_file(&PathBuf::from("src/oat.l"));
        self.nfa.borrow_mut().construct();
    }

    /// Constructs the DFA by setting the NFA for the DFA and performing the DFA construction.
    pub fn construct_dfa(&mut self) {
        self.dfa.borrow_mut().set_nfa(self.nfa.clone());
        self.dfa.borrow_mut().construct_dfa();
    }

    /// Minimizes the DFA.
    ///
    /// Currently a placeholder method for DFA minimization logic.
    pub fn minimize_dfa(&mut self) {}

    /// Lexes the input string using the constructed DFA.
    ///
    /// # Returns
    ///
    /// A vector of tuples where each tuple contains the token type and its corresponding lexeme.
    pub fn lex(&mut self) -> Vec<(String, String)> {
        self.dfa.borrow_mut().lex(&self.input)
    }
}
