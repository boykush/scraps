FROM rust:1.87

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}