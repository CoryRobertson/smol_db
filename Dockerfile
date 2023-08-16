FROM rust:1.71.1 as builder
COPY . .
RUN cargo build --package smol_db_server --release --features logging,statistics

FROM debian:bookworm
COPY --from=builder /target/release/smol_db_server ./smol_db_server
EXPOSE 8222
VOLUME ["/data"]
CMD ["./smol_db_server"]
