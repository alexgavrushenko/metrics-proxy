FROM debian:stable-slim as runner
RUN groupadd metrics && useradd -m -d /app -g metrics metrics
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl iproute2 \
    curl && \
    rm -rf /var/lib/apt/lists/*
USER metrics

FROM rust:1.40 as builder
ARG CARGO_BUILD_ARGS=""
ADD . /code
WORKDIR /code
RUN cargo build ${CARGO_BUILD_ARGS}

FROM runner
ARG BUILD_MODE="debug"
COPY --from=builder --chown=metrics:metrics /code/target/${BUILD_MODE}/metrics-proxy /app/metrics-proxy

