FROM rust:latest AS builder

RUN cargo install cargo-binstall
RUN cargo binstall dioxus-cli --no-confirm

WORKDIR /app/

COPY . /app

WORKDIR /app/backend
RUN cargo build --release

WORKDIR /app/frontend
RUN dx bundle --platform web --release

FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/target/dx/frontend/release/web/public /app/public
COPY --from=builder /app/target/release/backend /app/backend

# expose the port 80
EXPOSE 80

CMD ["/app/backend"]
