# Build:
# docker build -t simplicity-webide .
#
# Run:
# docker run -d -p 8080:80 --name simplicity-webide simplicity-webide

# Stage 1: Builder
# This stage installs all dependencies and builds the application.
FROM debian:bookworm-slim AS builder

# Install essential build tools, clang, and nodejs for sass
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    clang-16 \
    llvm-16 \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    nodejs \
    npm && \
    rm -rf /var/lib/apt/lists/*

# Install Rust and the wasm32 target
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && \
    rustup target add wasm32-unknown-unknown

# Install Trunk and Dart Sass
RUN cargo install trunk && \
    npm install -g sass

# Set environment variables for the Wasm C compiler, mimicking the Nix setup.
# This tells Rust's build scripts to use clang-16 when compiling C code for Wasm.
ENV CC_wasm32_unknown_unknown=clang-16
ENV AR_wasm32_unknown_unknown=llvm-ar-16
ENV CFLAGS_wasm32_unknown_unknown="-I/usr/lib/clang/16/include"

# Copy the application source code
WORKDIR /app
COPY . .

# Build the application WITHOUT the fix-links.sh script (for local deployment)
RUN trunk build --release

# Stage 2: Final Image
# This stage creates a minimal image to serve the built static files.
FROM nginx:1.27-alpine-slim

# Copy custom nginx configuration for proper routing
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Copy the built assets from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 80 and start Nginx
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

