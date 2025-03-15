use lrlex::CTLexerBuilder;

fn main() {
    CTLexerBuilder::new()
        .lexer_in_src_dir("oat.l")
        .unwrap()
        .build()
        .unwrap();
}
