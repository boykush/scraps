FROM debian:bookworm-slim

# Copy pre-built binary from build context
COPY scraps /usr/local/bin/scraps