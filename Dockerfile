FROM rust:1.68.0

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}
