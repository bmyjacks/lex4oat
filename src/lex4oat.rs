use crate::nfa::Nfa;
use std::path::PathBuf;

pub struct Lex4Oat {
    input: String,
    tokens: Vec<(String, String)>,
}

impl Lex4Oat {
    pub fn new(input: String) -> Lex4Oat {
        Lex4Oat {
            input,
            tokens: Vec::new(),
        }
    }

    pub fn construct_nfa(&mut self) {
        let mut nfa = Nfa::new();
        nfa.add_keywords_from_file(&PathBuf::from("src/oat.l"));
        nfa.construct();
    }
    pub fn construct_dfa(&mut self) {}
    pub fn minimize_dfa(&mut self) {}
    pub fn lex(&mut self) {}
}
