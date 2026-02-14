# Multi-stage build for smaller final image
FROM debian:bookworm-slim AS downloader

ARG VERSION

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && curl -sL "https://github.com/boykush/scraps/releases/download/v${VERSION}/scraps-x86_64-unknown-linux-gnu.tar.gz" | tar xz -C /usr/local/bin/ \
    && chmod +x /usr/local/bin/scraps

FROM debian:bookworm-slim

COPY --from=downloader /usr/local/bin/scraps /usr/local/bin/scraps