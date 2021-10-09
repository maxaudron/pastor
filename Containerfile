# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM kube.cat/cocainefarm/rust:1.55.0 AS chef
WORKDIR /work

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /work/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM scratch

COPY --from=builder /work/target/release/pastor /usr/local/bin/pastor

VOLUME /storage /templates
ENV ROCKET_STORAGE_DIR=/storage

CMD ["/usr/local/bin/pastor"]
