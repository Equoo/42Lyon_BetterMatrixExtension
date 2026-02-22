# ===== BUILD STAGE ====
FROM rust:latest AS builder

# Install PostgreSQL 16 + TimescaleDB 2 (like timescale/timescaledb:latest-pg16)
RUN apt-get update && \
    apt-get install -y wget gnupg lsb-release && \
    # Add PostgreSQL official repo
    echo "deb [signed-by=/usr/share/keyrings/pgdg.gpg] http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" \
        > /etc/apt/sources.list.d/pgdg.list && \
    wget -qO- https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/pgdg.gpg && \
    # Add TimescaleDB repo
    echo "deb [signed-by=/usr/share/keyrings/timescale.gpg] https://packagecloud.io/timescale/timescaledb/debian/ $(lsb_release -cs) main" \
        > /etc/apt/sources.list.d/timescaledb.list && \
    wget -qO- https://packagecloud.io/timescale/timescaledb/gpgkey | gpg --dearmor -o /usr/share/keyrings/timescale.gpg && \
    apt-get update && \
    apt-get install -y postgresql-16 timescaledb-2-postgresql-16 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy source
COPY ./server/ .
COPY ./database.sql .

# Initialize PostgreSQL, enable TimescaleDB, and import database.sql
USER postgres
RUN /usr/lib/postgresql/16/bin/initdb -D /var/lib/postgresql/data && \
    echo "shared_preload_libraries = 'timescaledb'" >> /var/lib/postgresql/data/postgresql.conf && \
    /usr/lib/postgresql/16/bin/pg_ctl -D /var/lib/postgresql/data -o "-c listen_addresses=''" -w start && \
    /usr/lib/postgresql/16/bin/psql --dbname=postgres -c "CREATE DATABASE clusters_db;" && \
    /usr/lib/postgresql/16/bin/psql --dbname=clusters_db -c "CREATE EXTENSION IF NOT EXISTS timescaledb;" && \
    /usr/lib/postgresql/16/bin/psql --dbname=clusters_db -f /usr/src/app/database.sql && \
    /usr/lib/postgresql/16/bin/pg_ctl -D /var/lib/postgresql/data stop
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
COPY --from=builder /usr/src/app/target/release/server .

# Run the binary
CMD ["./server"]


