//! Build script for generating the compile-time lexer using lrlex.
//!
//! This script uses lrlex's CTLexerBuilder to compile the token definitions
//! from the oat.l file located in the source directory. Errors during the build
//! process will cause a panic.

use lrlex::CTLexerBuilder;

/// The entry point for the build script.
///
/// This function initializes a CTLexerBuilder, specifies the lexer definition file,
/// and builds the compile-time lexer. Any errors during file access or lexer generation
/// will cause the build process to panic.
fn main() {
    // Create a new compile-time lexer builder.
    CTLexerBuilder::new()
        // Set the lexer definition file from the source directory.
        .lexer_in_src_dir("oat.l")
        .unwrap() // Panic if the lexer file cannot be read or processed.
        // Build the compile-time lexer.
        .build()
        .unwrap(); // Panic if the lexer construction fails.
}
