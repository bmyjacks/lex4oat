//! A simple Oat language lexer that demonstrates the use of two lexer implementations
//! (library-based and hand-made) and compares their tokenization results. The program
//! reads an input source file, processes it with both lexers, compares the output, and
//! prints tokens or error messages accordingly.

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

/// Command line arguments for the Oat language lexer.
///
/// This structure is used to configure the lexer by providing the input file.
/// The default input file is set to `a.oat`.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets input Oat source file.
    #[arg(value_name = "INPUT", default_value = "a.oat")]
    source_file: PathBuf,
}

/// The main entry point of the lexer application.
///
/// It initiates logging, parses command line arguments, reads the input file, and
/// processes it using both the library lexer and the hand-made lexer. The tokens produced
/// by both lexers are compared; if they match, the tokens are printed with proper color
/// formatting. Otherwise, an error is logged and the application exits.
fn main() {
    // Initialize the logger.
    env_logger::init();
    info!("Starting up");

    // Parse command line arguments.
    info!("Parsing arguments...");
    let args = Args::parse();
    info!("Parsed arguments: {:#?}", args);

    // Read the source file, logging its name in yellow.
    info!(
        "Reading source file {}",
        args.source_file.display().to_string().yellow()
    );
    let input = std::fs::read_to_string(&args.source_file).unwrap_or_else(|err| {
        // Log error message with colored output and exit the process.
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

    // Library lexer section.
    info!("Parsing source file using library lexer...");
    let mut lib_lexer = LibLex4Oat::new(input.to_owned());
    lib_lexer.lex();
    let lib_tokens = lib_lexer.tokens();
    info!("Done parsing library lexer");

    // Handmade lexer section.
    info!("Parsing source file using hand-baked parser...");
    let mut hand_lexer = lex4oat::Lex4Oat::new(input.to_owned());
    hand_lexer.construct_nfa();
    hand_lexer.construct_dfa();
    hand_lexer.minimize_dfa();
    let hand_tokens = hand_lexer.lex();
    info!("Done parsing hand-made lexer");

    // Compare tokens from both lexer implementations.
    info!("Checking result...");
    let mut check = true;
    if lib_tokens.len() != hand_tokens.len() {
        // Log error and exit if token count does not match.
        error!("Length of tokens doesn't match");
        process::exit(1);
    } else {
        // Iterate through tokens and compare each pair.
        for i in 0..hand_tokens.len() {
            if lib_tokens[i].0 != hand_tokens[i].0 || lib_tokens[i].1 != hand_tokens[i].1 {
                warn!("Mismatched tokens found: {}", lib_tokens[i].1);
                check = false;
            }
        }
    }

    // If tokens match, print them with a green success message.
    if check {
        info!("{}", "Result matched".green());
        for (typ, token) in hand_tokens {
            println!("{:<15} {}", typ, token);
        }
        info!("Done, good day!");
    }
}
