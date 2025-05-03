FROM rust:1.85

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}