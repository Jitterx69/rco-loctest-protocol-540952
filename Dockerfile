# --- Stage 1: Build Core ---
FROM rust:1.77-alpine3.19 AS builder

RUN apk add --no-cache musl-dev gcc make perl-dev clang-dev llvm-dev nasm

WORKDIR /usr/src/rco-protocol
COPY . .

# Build the assembly-optimized workspace
RUN cargo build --release --workspace

# --- Stage 2: Runtime ---
FROM alpine:3.19

# Add required runtimes (Node.js and OpenJDK for polyglot peak)
RUN apk add --no-cache \
    libgcc \
    libstdc++ \
    nodejs \
    npm \
    openjdk21-jre-headless \
    tpm2-tss

WORKDIR /opt/rco

# Copy native artifacts
COPY --from=builder /usr/src/rco-protocol/target/release/*.so /usr/local/lib/
COPY --from=builder /usr/src/rco-protocol/dist/include /usr/local/include/rco/

# Set up environment for the Sovereign Manifold
ENV LD_LIBRARY_PATH=/usr/local/lib
ENV RCO_TPM_DEVICE=/dev/tpmrm0

# Default entrypoint for a sovereign node
ENTRYPOINT ["/usr/local/lib/librco_core.so"]
