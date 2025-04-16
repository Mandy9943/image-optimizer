FROM rust:latest

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy all the source code and static files
COPY . .

# Build the application
RUN cargo build --release

# Create required directories and ensure proper permissions
RUN mkdir -p /app/static/optimized && \
    chmod -R 777 /app/static

# Expose the port that the application listens on
EXPOSE 3655

# Set environment variables for more verbose logging
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

# Run the application
CMD ["/app/target/release/images-optimizer"] 