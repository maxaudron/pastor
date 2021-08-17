# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM docker.io/rust:1.52-alpine as cargo-build

RUN rustup default nightly && rustup update

WORKDIR /work

COPY . .

RUN apk add --no-cache musl-dev
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM docker.io/alpine:latest

COPY --from=cargo-build /work/target/x86_64-unknown-linux-musl/release/pastor /usr/local/bin

RUN apk add openssl
RUN adduser pastor -D

RUN mkdir /storage /templates && chown pastor templates storage
VOLUME /storage /templates

ENV ROCKET_STORAGE_DIR /storage

USER pastor

CMD ["/usr/local/bin/pastor"]
