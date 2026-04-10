FROM rust:latest

RUN cargo install cargo-watch

WORKDIR /app/

COPY . /app

WORKDIR /app/backend

RUN cargo build

CMD ["cargo", "watch", "-x", "run"]
