# syntax=docker/dockerfile:1

# --- Build stage: compile Rust application ---
FROM rust:1-alpine AS builder
WORKDIR /app/label-print-server

# Install Alpine build toolchain and musl headers
RUN apk add --no-cache build-base musl-dev pkgconfig openssl-dev zlib-dev

# Copy manifests first to leverage Docker cache for dependencies
COPY label-print-server/Cargo.toml label-print-server/Cargo.lock ./
# Pre-build empty main to cache deps
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release || true

# Copy real sources and Typst template
RUN rm -rf src
COPY label-print-server/src ./src
COPY label-print-server/99012.typ ./99012.typ

# Build release binary
RUN cargo build --release

# --- Runtime stage: based on typst image ---
FROM ghcr.io/typst/typst:latest
# Install fontconfig for fonts and cups-client for the `lp` command
RUN apk add --no-cache fontconfig cups-client
# Create directories expected by the app and for fonts
RUN mkdir -p /app/label-print-server /usr/share/fonts/truetype/custom
WORKDIR /app/label-print-server

# Copy the compiled binary
COPY --from=builder /app/label-print-server/target/release/label-print-server /usr/local/bin/label-print-server

# Copy the Typst template to the same absolute path used during compile time
COPY label-print-server/99012.typ /app/label-print-server/99012.typ

# Copy custom font and refresh font cache
COPY ["Please write me a song.ttf", "/usr/share/fonts/truetype/custom/"]
RUN fc-cache -f -v

# Configure and run server
ENV HOST=0.0.0.0 \
    PORT=8080 \
    PRINTER=DYMO_LabelWriter_450
EXPOSE 8080

#For debugging purposes I will use the following entrypoint:
#ENTRYPOINT ["tail", "-f", "/dev/null"]
ENTRYPOINT ["/usr/local/bin/label-print-server"]