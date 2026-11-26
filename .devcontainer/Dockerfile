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

# cargo-deny for `cargo xtask supply-chain` (ADR-0039). Same version as in CI.
ARG CARGO_DENY_VERSION=0.22.1
RUN set -eu; \
    case "$TARGETARCH" in \
      amd64) arch=x86_64; sum=13f7e825e9cb811c2f8de63f29b0f45851dfb53004029f0f2501af524728364d ;; \
      arm64) arch=aarch64; sum=0bac0838a1653ce693e4602b8634d15ca179edd2198e438af9c35ab1c551d509 ;; \
      *) echo "unsupported architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    name="cargo-deny-${CARGO_DENY_VERSION}-${arch}-unknown-linux-musl"; \
    curl -fsSL -o "/tmp/$name.tar.gz" \
      "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/$name.tar.gz"; \
    echo "$sum  /tmp/$name.tar.gz" | sha256sum -c -; \
    tar -xzf "/tmp/$name.tar.gz" -C /tmp; \
    mv "/tmp/$name/cargo-deny" /usr/local/cargo/bin/; \
    rm -rf "/tmp/$name" "/tmp/$name.tar.gz"; \
    cargo-deny --version

RUN apt-get update \
    && apt-get install -y --no-install-recommends postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Node and pnpm for the frontend (ADR-0038, ADR-0043). The versions match
# .node-version and the packageManager field of web/package.json; update them
# together. Node no longer ships corepack, so pnpm is installed at its pinned version.
COPY --from=node:26.13.1-trixie-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:26.13.1-trixie-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
ARG PNPM_VERSION=12.23.0
RUN ln -s ../lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s ../lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx \
    && npm install --global --no-fund --no-audit "pnpm@${PNPM_VERSION}" \
    && node --version && pnpm --version

# The system libraries Playwright's Chromium and Firefox need (ADR-0043). The browsers
# themselves are downloaded on demand into the user's cache: `pnpm exec playwright
# install chromium firefox` in web/. The version matches @playwright/test.
ARG PLAYWRIGHT_VERSION=1.65.1
RUN npx --yes "playwright@${PLAYWRIGHT_VERSION}" install-deps chromium firefox \
    && rm -rf /var/lib/apt/lists/* /root/.npm

RUN useradd --create-home --uid 1000 dev && chown -R dev /usr/local/cargo
USER dev
