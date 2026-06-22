# ---- build stage ----
FROM rust:1.89-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p soldecoders-node

# ---- runtime stage ----
FROM debian:bookworm-slim AS runtime
RUN useradd -u 10001 -m appuser
COPY --from=builder /app/target/release/soldecoders-node /usr/local/bin/soldecoders-node
USER 10001
EXPOSE 8080
ENTRYPOINT ["soldecoders-node"]
CMD ["serve"]
