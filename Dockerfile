FROM rust:1.83

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}