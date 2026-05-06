FROM rust:latest AS builder

RUN cargo install cargo-binstall
RUN cargo binstall dioxus-cli --no-confirm

WORKDIR /app/

COPY . /app

WORKDIR /app/backend
RUN cargo build --release

WORKDIR /app/frontend
RUN dx bundle --platform web --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/dx/frontend/release/web/public /app/backend/public
COPY --from=builder /app/target/release/backend /app/backend/backend

# expose the port 80
EXPOSE 80
EXPOSE 8090
EXPOSE 9090

CMD ["/app/backend/backend"]
