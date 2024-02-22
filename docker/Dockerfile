FROM rust:1.76-slim-bullseye

ARG HOMEDIR=/home/build
ARG USER_UID=11000
RUN useradd build \
      --uid $USER_UID \
      --password '!' \
      --shell /bin/bash \
      --home-dir $HOMEDIR \
      --create-home

RUN apt-get update && \
    apt-get install -y build-essential llvm-dev libclang-dev clang musl-dev musl-tools && \
    apt-get clean

RUN rustup component add rustfmt clippy

USER build
WORKDIR $HOMEDIR

ENV CARGO_HOME=$HOMEDIR/cargo
ENV CARGO_TARGET_DIR=$HOMEDIR/target
RUN mkdir -p $HOMEDIR/workspace && \
    mkdir -p "${CARGO_HOME}" && \
    mkdir -p "${CARGO_TARGET_DIR}"
VOLUME $HOMEDIR/workspace
VOLUME $HOMEDIR/cargo
VOLUME $HOMEDIR/target

WORKDIR $HOMEDIR/workspace
