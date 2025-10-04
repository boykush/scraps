# Multi-stage build for smaller final image
FROM rust:1.90-slim-bookworm AS builder

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}

FROM debian:bookworm-slim

# Copy the binary from builder stage
COPY --from=builder /usr/local/cargo/bin/scraps /usr/local/bin/scraps