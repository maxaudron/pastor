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
COPY --from=cargo-build /work/templates /templates

ENV ROCKET_TEMPLATE_DIR /templates

RUN apk add openssl
RUN adduser pastor -D

USER pastor

CMD ["/usr/local/bin/pastor"]
