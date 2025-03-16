use crate::dfa::Dfa;
use crate::nfa::Nfa;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Lex4Oat {
    input: String,
    tokens: Vec<(String, String)>,
    nfa: Rc<RefCell<Nfa>>,
    dfa: Rc<RefCell<Dfa>>,
}

impl Lex4Oat {
    pub fn new(input: String) -> Lex4Oat {
        let nfa = Rc::new(RefCell::new(Nfa::new()));
        let dfa = Rc::new(RefCell::new(Dfa::new()));
        Lex4Oat {
            input,
            tokens: Vec::new(),
            nfa,
            dfa,
        }
    }

    pub fn construct_nfa(&mut self) {
        self.nfa
            .borrow_mut()
            .add_keywords_from_file(&PathBuf::from("src/oat.l"));
        self.nfa.borrow_mut().construct();
    }

    pub fn construct_dfa(&mut self) {
        self.dfa.borrow_mut().set_nfa(self.nfa.clone());
        self.dfa.borrow_mut().construct_dfa();
    }

    pub fn minimize_dfa(&mut self) {}
    pub fn lex(&mut self) {
        let tokens = self.dfa.borrow_mut().lex(&self.input);

        for token in tokens {
            println!("{}", token);
        }
    }
}
