// Use colored for output colorization and log for logging errors.
use colored::Colorize;
use log::error;
// Import lrlex and lrpar for lexer definition and lexeme handling.
use lrlex::{lrlex_mod, LexerDef};
use lrpar::{Lexeme, Lexer, NonStreamingLexer};

// Generates modules and lexer definitions from the oat.l file.
lrlex_mod!("oat.l");

/// A library lexer for the Oat language using lrlex and lrpar.
///
/// This struct holds the input source code and tokens extracted from lexical analysis.
pub struct LibLex4Oat {
    /// The input source code as a string.
    input: String,
    /// A vector of tuples mapping token names to their lexemes.
    tokens: Vec<(String, String)>,
}

impl LibLex4Oat {
    /// Creates a new instance of `LibLex4Oat` with the given input.
    ///
    /// # Arguments
    ///
    /// * `input` - A string containing the source code to lex.
    ///
    /// # Returns
    ///
    /// A new instance of `LibLex4Oat`.
    pub fn new(input: String) -> Self {
        LibLex4Oat {
            input,
            tokens: Vec::new(),
        }
    }

    /// Returns a reference to the tokens vector.
    ///
    /// # Returns
    ///
    /// A reference to a vector containing tuples of token name and token lexeme.
    pub fn tokens(&self) -> &Vec<(String, String)> {
        &self.tokens
    }

    /// Performs lexical analysis on the input source code.
    ///
    /// This method uses `lrlex` to generate a lexer definition from `oat.l` and processes
    /// the input code. Tokens are extracted by iterating over lexemes and are stored
    /// along with their corresponding token names. In case of any lexer error, the error is logged.
    pub fn lex(&mut self) {
        let lexerdef = oat_l::lexerdef();
        let lexer = lexerdef.lexer(&self.input);

        // Iterate through each lexeme generated by the lexer.
        for lexeme in lexer.iter() {
            match lexeme {
                Ok(lexeme) => {
                    // Get the lexeme slice from the input.
                    let span = lexer.span_str(lexeme.span());
                    // Retrieve token id and name based on the lexer definition.
                    let tok_id = lexeme.tok_id();
                    let tok_name = lexerdef.get_rule_by_id(tok_id).name().unwrap();
                    // Store the token name and its lexeme.
                    self.tokens.push((tok_name.to_string(), span.to_string()));
                }
                // Log the error and break the loop if any lexeme results in an error.
                Err(err) => {
                    error!("Library lexer error: {}", err.to_string().red());
                    break;
                }
            }
        }
    }
}
