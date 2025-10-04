FROM rust:1.90

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}