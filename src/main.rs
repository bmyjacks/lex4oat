mod dfa;
mod lex4oat;
mod liblex4oat;
mod nfa;
mod node;

use crate::liblex4oat::LibLex4Oat;
use clap::arg;
use clap::Parser;
use std::path::PathBuf;

/// A useful Oat language lexer
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets input Oat source file
    #[arg(value_name = "INPUT", default_value = "test.oat")]
    source_file: PathBuf,

    #[arg(value_name = "OUTPUT type", long = "emit", default_value = "tokens",
        value_parser = ["tokens", "nfa", "dfa"])]
    emit: String,

    /// Use verbose output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();
    let input = std::fs::read_to_string(args.source_file).expect("Failed to read input file");

    // Library lexer
    // let mut lib_lexer = LibLex4Oat::new(input.to_owned());
    // lib_lexer.lex();
    // 
    // let tokens = lib_lexer.tokens();
    // for (name, span) in tokens {
    //     println!("{:<10} {}", name, span);
    // }
    // println!();
    // End of library lexer

    // Handmade lexer
    let mut hand_lexer = lex4oat::Lex4Oat::new(input.to_owned());
    hand_lexer.construct_nfa();
    hand_lexer.construct_dfa();
    hand_lexer.minimize_dfa();
    hand_lexer.lex();
}
