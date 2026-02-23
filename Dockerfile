# ===== BUILD STAGE ====
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy source
COPY . .

USER root

# Build the Rust project
RUN cargo build --release

# ===== FINAL STAGE =====
FROM rust:latest

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/BetterMatrixServer .

# Run the binary
CMD ["./BetterMatrixServer"]


