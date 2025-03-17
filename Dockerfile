FROM rust:1.85

RUN rustup default nightly

WORKDIR /lex4oat
COPY . .


RUN cargo install --path .

ENTRYPOINT ["lex4oat"]