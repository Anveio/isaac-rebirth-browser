# Antithesis image for the engine DST harness (docs/dst.md). Build from the
# repository root:  docker build -f antithesis/Dockerfile -t rebirth-dst .
# The binary embeds data compiled from the private original resources; pushing
# it to any external registry, Antithesis included, needs the owner's approval.
#
# INSTRUMENT=1 adds Antithesis coverage instrumentation and requires the
# platform's libvoidstar.so in the build context at antithesis/libvoidstar.so.
# rust-toolchain.toml selects the compiler; a matching base image only avoids
# a second download.
FROM rust:1.99-bookworm AS build
ARG INSTRUMENT=0
RUN apt-get update && apt-get install -y --no-install-recommends python3 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY rust-toolchain.toml rust-toolchain.toml
RUN rustup toolchain install
COPY engine engine
COPY tools/foundation/item-catalog.py tools/foundation/item-catalog.py
COPY web/*.json web/
COPY references/isaac/rebirth/resources references/isaac/rebirth/resources
COPY antithesis antithesis
RUN cargo build --manifest-path engine/Cargo.toml --locked --release --target wasm32-unknown-unknown --lib
RUN if [ "$INSTRUMENT" = 1 ]; then \
      cp antithesis/libvoidstar.so /usr/lib/libvoidstar.so && \
      export RUSTFLAGS="-Ccodegen-units=1 -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=3 -Cllvm-args=-sanitizer-coverage-trace-pc-guard -Clink-args=-Wl,--build-id -L/usr/lib -lvoidstar"; \
    fi && \
    cargo build --manifest-path engine/dst/Cargo.toml --locked --release && \
    mkdir -p /out/lib /symbols && \
    cp engine/dst/target/release/rebirth-dst /out/ && \
    cp engine/dst/target/release/rebirth-dst /symbols/ && \
    if [ "$INSTRUMENT" = 1 ]; then cp /usr/lib/libvoidstar.so /out/lib/; fi

FROM debian:bookworm-slim
COPY --from=build /out/lib/ /usr/lib/
COPY --from=build /symbols /symbols
COPY --from=build /out/rebirth-dst /usr/local/bin/rebirth-dst
COPY --from=build /src/engine/target/wasm32-unknown-unknown/release/rebirth_cleanroom.wasm /opt/rebirth/engine.wasm
COPY antithesis/test /opt/antithesis/test
COPY antithesis/entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
