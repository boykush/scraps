FROM rust:1.86

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}