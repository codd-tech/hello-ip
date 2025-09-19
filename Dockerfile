FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/hello-ip /usr/local/bin/hello-ip

USER nonroot:nonroot

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/hello-ip"]