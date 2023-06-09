FROM mcr.microsoft.com/cbl-mariner/base/core:2.0 as builder

ARG RUST_VERSION

# Rust install and app build
RUN tdnf install -yq tar ca-certificates \
    && echo -e "Finished tar and ca-certificates" \
    && tdnf install -yq build-essential \
    && echo -e "finished build-essential" \
    && curl "https://static.rust-lang.org/dist/rust-$RUST_VERSION-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.gz \
    && tar -C /tmp -xzf /tmp/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.gz \
    && sh /tmp/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu/install.sh \
    && rm -rf /tmp/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu /tmp/rust-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.gz

WORKDIR /usr/src/aks-egress-checker
COPY . .
RUN cargo build --release


# Deployable image
FROM mcr.microsoft.com/cbl-mariner/base/core:2.0-nonroot

COPY --from=builder /usr/src/aks-egress-checker/target/release/aks-egress-checker /usr/local/bin/aks-egress-checker
COPY --from=builder /usr/src/aks-egress-checker/egress-data/** /etc/egress-data

CMD [ "aks-egress-checker" ]