# Toolchain of the devcontainer (ADR-0038). Versions match rust-toolchain.toml and the
# hooks (ADR-0041); update them together.
FROM rust:1.100.0

RUN rustup component add rustfmt clippy

# Betterleaks for the git hooks, so they need no Docker inside the container.
ARG BETTERLEAKS_VERSION=1.10.1
ARG TARGETARCH
RUN set -eu; \
    case "$TARGETARCH" in \
      amd64) arch=x64; sum=c174b42d7aa60caf58c0c3e555c0e4bed01fe6b0d99a412bce777b5af4ce913e ;; \
      arm64) arch=arm64; sum=62a63199f9f61a6b5e5a6f58f048493db20fbc8c770014f4d903a4d97886e1bf ;; \
      *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    file="betterleaks_${BETTERLEAKS_VERSION}_linux_${arch}.tar.gz"; \
    curl -fsSL -o "/tmp/$file" \
      "https://github.com/betterleaks/betterleaks/releases/download/v${BETTERLEAKS_VERSION}/$file"; \
    echo "$sum  /tmp/$file" | sha256sum -c -; \
    tar -xzf "/tmp/$file" -C /usr/local/bin betterleaks; \
    rm "/tmp/$file"; \
    betterleaks version

RUN apt-get update \
    && apt-get install -y --no-install-recommends postgresql-client \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 1000 dev && chown -R dev /usr/local/cargo
USER dev
