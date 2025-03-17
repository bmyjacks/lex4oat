# lex4oat

lex4oat is a Rust-based lexer for the Oat programming language. It features two lexing approaches:

- **Library Lexer:** Uses [`lrlex`](src/liblex4oat.rs) and [`lrpar`](src/liblex4oat.rs) to tokenize the source.
- **Handcrafted Lexer:** Builds an NFA and converts it to a DFA (see [`src/lex4oat.rs`](src/lex4oat.rs), [`src/nfa.rs`](src/nfa.rs), and [`src/dfa.rs`](src/dfa.rs)) to perform tokenization.

## Features

- Tokenizes Oat source files (e.g. [`a.oat`](a.oat))
- Demonstrates lexer construction using state machines (NFA & DFA)
- Provides both library-based and hand-made lexing techniques
- Uses [`clap`](Cargo.toml) for command-line argument parsing and [`env_logger`](Cargo.toml) for logging

## Getting Started

### Build

```shell
cargo build
```

### Run

```shell
cargo run -- -f a.oat
```

### Docker
```shell
docker buildx build . -t lex4oat:1

docker run -it --rm lex4oat:1 test/[].oat
```

### Testing & CI

The project includes a GitHub Actions workflow that builds and tests the project.
