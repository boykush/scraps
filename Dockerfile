FROM rust:1.89

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}