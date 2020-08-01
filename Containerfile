# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:1.41 as cargo-build

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup default nightly && rustup update
RUN rustup target add x86_64-unknown-linux-musl --toolchain=nightly

WORKDIR /work

COPY . .

RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

COPY --from=cargo-build /work/target/x86_64-unknown-linux-musl/release/pastor /usr/local/bin

RUN apk add openssl
RUN adduser pastor -D

RUN mkdir /storage /templates && chown pastor templates storage
VOLUME /storage /templates

ENV ROCKET_TEMPLATE_DIR /templates
ENV ROCKET_STORAGE_DIR /storage

USER pastor

CMD ["/usr/local/bin/pastor"]
