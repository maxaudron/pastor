# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM kube.cat/cocainefarm/rust:1.57.0 AS chef

WORKDIR /work
RUN wget http://ftp.astron.com/pub/file/file-5.41.tar.gz && \
    tar xf file-5.41.tar.gz

WORKDIR /work/file-5.41

RUN apk add autoconf libtool automake make && \
    SH_LIBTOOL='/usr/share/build-1/libtool' autoreconf -f -i && \
    ./configure --prefix=/usr --datadir=/usr/share --enable-static --disable-shared && \
    make -j32 && make install && cd .. && rm -rf /work/file-5.41

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
