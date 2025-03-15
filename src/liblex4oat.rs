use lrlex::{lrlex_mod, LexerDef};
use lrpar::{Lexeme, Lexer, NonStreamingLexer};

lrlex_mod!("oat.l");

pub struct LibLex4Oat {
    input: String,
    tokens: Vec<(String, String)>,
}

impl LibLex4Oat {
    pub fn new(input: String) -> Self {
        LibLex4Oat {
            input,
            tokens: Vec::new(),
        }
    }

    pub fn tokens(&self) -> &Vec<(String, String)> {
        &self.tokens
    }

    pub fn lex(&mut self) {
        let lexerdef = oat_l::lexerdef();
        let lexer = lexerdef.lexer(&self.input);

        for lexeme in lexer.iter() {
            match lexeme {
                Ok(lexeme) => {
                    let span = lexer.span_str(lexeme.span());
                    let tok_id = lexeme.tok_id();
                    let tok_name = lexerdef.get_rule_by_id(tok_id).name().unwrap();
                    self.tokens.push((tok_name.to_string(), span.to_string()));
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
