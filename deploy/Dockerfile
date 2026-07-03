# Multi-stage build for dft2dxf-sidecar

FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p dft2dxf-sidecar

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/dft2dxf-sidecar /usr/local/bin/dft2dxf-sidecar
ENV SIDECAR_HOST=0.0.0.0
ENV SIDECAR_PORT=8080
ENV WORKER_CONCURRENCY=4
EXPOSE 8080
ENTRYPOINT ["dft2dxf-sidecar"]
