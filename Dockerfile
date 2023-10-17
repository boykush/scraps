FROM rust:latest

ARG INSTALL_VERSION

RUN cargo install scraps --version ${INSTALL_VERSION}