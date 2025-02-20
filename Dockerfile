FROM rust:1.84

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}