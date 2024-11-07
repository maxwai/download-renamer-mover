FROM rust:1.82.0 as builder

# create a new empty shell project
RUN USER=root cargo new --bin download-renamer-mover
WORKDIR /download-renamer-mover

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Now that the dependency is built, copy your source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/download_renamer_mover*
RUN cargo install --path .

FROM debian:bookworm-slim

WORKDIR /download-renamer-mover

RUN apt-get update && apt install -y openssl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# initialize user as needed
RUN useradd -u 1001 -s /bin/sh abc

# copy entrypoint
COPY ./entrypoint.sh .

# Fix permissions
RUN chmod +x entrypoint.sh

# copy log config
COPY ./log4rs.yml .

# copy the build artifact from the build stage
COPY --from=builder /usr/local/cargo/bin/download-renamer-mover /usr/local/bin/download-renamer-mover

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

ENTRYPOINT ./entrypoint.sh