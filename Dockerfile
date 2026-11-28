# The one image of Iasion (ADR-0040, decisions 1 to 3): the backend binary and the
# built frontend, which the backend serves, on a distroless runtime as a non-root user.
# The versions match .node-version, web/package.json and rust-toolchain.toml; the base
# images are pinned by digest too, which Dependabot updates with the tag (ADR-0039).

FROM node:26.13.1-trixie-slim@sha256:abe7e5a82b61a8ef67b315b85497c9e65173387ccf2f4a029248184b8e020be5 AS web
# Node no longer ships corepack; pnpm at the version web/package.json pins (ADR-0043).
ARG PNPM_VERSION=12.23.0
RUN npm install --global --no-fund --no-audit "pnpm@${PNPM_VERSION}"
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm run build

# The workspace with an empty source for every crate: all that compiling the
# dependencies needs. Its content changes only with a manifest or the lockfile, so the
# layer below that compiles the dependencies stays cached while only code changes.
# The crates are the workspace members of Cargo.toml.
FROM rust:1.100.0-slim-trixie@sha256:3b199fd6db7d709274df56565b8f4003f9e30d7bfcff9d53148c86053b7855e2 AS manifests
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY xtask/ xtask/
RUN mkdir /manifests && cp Cargo.toml Cargo.lock /manifests/ \
    && for crate in crates/* xtask; do \
        mkdir -p "/manifests/$crate/src" && cp "$crate/Cargo.toml" "/manifests/$crate/"; \
        if [ -f "$crate/src/main.rs" ]; then \
            echo 'fn main() {}' >"/manifests/$crate/src/main.rs"; \
        else \
            : >"/manifests/$crate/src/lib.rs"; \
        fi; \
    done

# Debian 13, like the runtime image below.
FROM rust:1.100.0-slim-trixie@sha256:3b199fd6db7d709274df56565b8f4003f9e30d7bfcff9d53148c86053b7855e2 AS server
# The toolchain of rust-toolchain.toml, without the components it lists for development.
ENV RUSTUP_TOOLCHAIN=1.100.0
WORKDIR /src
COPY --from=manifests /manifests ./
RUN cargo build --release --locked --package iasion-server
COPY . .
# Cargo compares modification times, and a real source can be older than the empty one
# compiled above, so every crate root counts as changed. `-c` creates no missing file.
RUN touch -c crates/*/src/lib.rs crates/*/src/main.rs xtask/src/main.rs
# The commit, compiled into the binary and shown by the app; CI passes the SHA. Declared
# here, so that a new commit does not recompile the dependencies.
ARG IASION_REVISION=dev
RUN cargo build --release --locked --package iasion-server \
    && cp target/release/iasion /iasion

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:29ee70762214fe8190cea04ef1c0a84a8365879e047ce8f911ae5a5ab2355fa2
COPY --from=server /iasion /iasion
COPY --from=web /src/web/dist /app/web
ENV IASION_WEB_DIR=/app/web
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/iasion"]
CMD ["serve"]
