FROM rust:1.88

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}