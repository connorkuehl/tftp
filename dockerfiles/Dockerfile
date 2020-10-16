# To produce a smaller image this Dockerfile contains two separate stages: in
# the first one all the build dependencies are installed tftp is built,
# while in the second one just the runtime dependencies are installed, with the
# binary built in the previous stage copied there.

#################
#  Build stage  #
#################

FROM rust:1.43 as build

# Build the dependencies in a separate step to avoid rebuilding all of them
# every time the source code changes. This takes advantage of Docker's layer
# caching, and it works by copying the Cargo.{toml,lock} with dummy source code
# and doing a full build with it.
RUN USER=root cargo new --bin tftp
WORKDIR ./build
COPY ./Cargo.toml ./Cargo.toml
RUN mkdir examples && \
    echo "fn main() {}" > examples/server.rs

RUN cargo fetch
RUN cargo build --example server

# Dependencies are now cached, copy the actual source code and do another full
# build. The touch on all the .rs files is needed, otherwise cargo assumes the
# source code didn't change thanks to mtime weirdness.
RUN rm -rf src server.rs

COPY .git .git
COPY examples examples/
COPY src src/
COPY artifacts artifacts/
RUN find examples -name "*.rs" -exec touch {} \;
RUN find src -name "*.rs" -exec touch {} \;
RUN find artifacts -name "*.rs" -exec touch {} \;

RUN cargo build --examples

##################
#  Output stage  #
##################

FROM debian:buster-slim AS output

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    git \
    libmagic1 \
    docker.io \
    ca-certificates \
    build-essential \
    gcc \
    pkg-config \
    libssl-dev

EXPOSE 6655

COPY --from=build /build/target/debug/examples/server /usr/local/bin
WORKDIR /usr/local/bin

CMD ["./server", "0.0.0.0:6655", "./artifacts"]
