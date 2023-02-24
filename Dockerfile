FROM rust:latest

# We need this stuff to build
RUN apt-get update && apt-get install -y libdbus-1-dev pkg-config
# Warm the crates.io cache
RUN cargo init --name temp && cargo add tokio && cargo build

# Build
COPY . /source/
WORKDIR /source/
RUN cargo build
