# Use the official Rust 1.74 image as the base image
FROM rust:1.74 as builder

# Set the working directory in the container
WORKDIR /app/src

# Install SQLx CLI
RUN cargo install sqlx-cli@0.7.3

# Install nvm and Node.js 18
RUN apt-get update && curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash 

ENV NVM_DIR=/root/.nvm
ENV NODE_VERSION=18.17.0
RUN . "$NVM_DIR/nvm.sh" && \
    nvm install ${NODE_VERSION} && \
    nvm use ${NODE_VERSION} && \
    nvm alias default ${NODE_VERSION}

ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

# Check Node.js
RUN node --version

# Copy the necessary files for SQLx migrations
COPY ./migrations ./migrations

# Create and apply the migration
RUN mkdir data && sqlx database create --database-url "sqlite:./data/data.db"
RUN sqlx migrate run --database-url "sqlite:./data/data.db"

# Copy the entire project to the container
COPY . .

# Run the tw:build script
RUN npm install && npm run tw:build

# Build the Rust project
RUN export DATABASE_URL="sqlite:./data/data.db" && cargo build --release

# Create a new image
FROM ubuntu:24.04 as runner
RUN apt-get update && \
    apt-get install openssl -y && \
    apt-get install -y ca-certificates

# Set the working directory in the container
WORKDIR /app

# Copy necessary files to run the binary
COPY --from=builder /app/src/target/release/axum-oauth-sample /app/axum-oauth-sample
COPY --from=builder /app/src/data /app/data
COPY --from=builder /app/src/public /app/public
COPY --from=builder /app/src/templates /app/templates

# Run the Rust project
CMD ["./axum-oauth-sample"]
