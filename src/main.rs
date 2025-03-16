mod dfa;
mod lex4oat;
mod liblex4oat;
mod nfa;
mod node;

use crate::liblex4oat::LibLex4Oat;
use clap::arg;
use clap::Parser;
use colored::Colorize;
use log::{error, info, warn};
use std::path::PathBuf;
use std::process;

/// A useful Oat language lexer
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets input Oat source file
    #[arg(value_name = "INPUT", default_value = "a.oat")]
    source_file: PathBuf,

    #[arg(value_name = "OUTPUT type", long = "emit", default_value = "tokens",
        value_parser = ["tokens", "nfa", "dfa"])]
    emit: String,
}

fn main() {
    env_logger::init();
    info!("Starting up");

    info!("Parsing arguments...");
    let args = Args::parse();
    info!("Parsed arguments: {:#?}", args);

    info!(
        "Reading source file {}",
        args.source_file.display().to_string().yellow()
    );
    let input = std::fs::read_to_string(&args.source_file).unwrap_or_else(|err| {
        error!(
            "Failed to read input file {}: {}",
            args.source_file.display().to_string().yellow(),
            err.to_string().red()
        );
        process::exit(1);
    });
    info!(
        "Done reading source file {}",
        args.source_file.display().to_string().yellow()
    );

    // Library lexer
    info!("Parsing source file using library lexer...");
    let mut lib_lexer = LibLex4Oat::new(input.to_owned());
    lib_lexer.lex();
    let lib_tokens = lib_lexer.tokens();
    info!("Done parsing library lexer");

    // Handmade lexer
    info!("Parsing source file using hand-baked parser...");
    let mut hand_lexer = lex4oat::Lex4Oat::new(input.to_owned());
    hand_lexer.construct_nfa();
    hand_lexer.construct_dfa();
    hand_lexer.minimize_dfa();
    let hand_tokens = hand_lexer.lex();
    info!("Done parsing hand-made lexer");

    info!("Checking result...");

    let mut check = true;
    for i in 0..hand_tokens.len() {
        if lib_tokens[i].0 != hand_tokens[i].0 || lib_tokens[i].1 != hand_tokens[i].1 {
            warn!("MISMATCH");
            check = false;
        }
    }

    if check {
        info!("{}", "Result matched".green());
        for (typ, token) in hand_tokens {
            println!("{:<15} {}", typ, token);
        }
    }

    info!("Done, good day!");
}
