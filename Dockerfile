FROM docker.io/rust:1.65.0-slim-buster as builder

WORKDIR /usr/src/aks-egress-checker
COPY . .
RUN cargo build --release


# Deployable image
FROM docker.io/ubuntu:jammy

RUN apt-get update \
    && apt-get upgrade -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/aks-egress-checker/target/release/aks-egress-checker /usr/local/bin/aks-egress-checker
COPY --from=builder /usr/src/aks-egress-checker/egress-data/** /etc/egress-data
CMD [ "aks-egress-checker" ]