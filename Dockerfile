# The one image of Iasion (ADR-0040, decisions 1 to 3): the backend binary and the
# built frontend, which the backend serves, on a distroless runtime as a non-root user.
# The versions match .node-version, web/package.json and rust-toolchain.toml.

FROM node:26.13.1-trixie-slim AS web
# Node no longer ships corepack; pnpm at the version web/package.json pins (ADR-0043).
ARG PNPM_VERSION=12.23.0
RUN npm install --global --no-fund --no-audit "pnpm@${PNPM_VERSION}"
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY web/ ./
RUN pnpm run build

FROM rust:1.100.0 AS server
# The toolchain of rust-toolchain.toml, without the components it lists for development.
ENV RUSTUP_TOOLCHAIN=1.100.0
# The commit, compiled into the binary and shown by the app; CI passes the SHA.
ARG IASION_REVISION=dev
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --package iasion-server \
    && cp target/release/iasion /iasion

FROM gcr.io/distroless/cc-debian13:nonroot@sha256:29ee70762214fe8190cea04ef1c0a84a8365879e047ce8f911ae5a5ab2355fa2
COPY --from=server /iasion /iasion
COPY --from=web /src/web/dist /app/web
ENV IASION_WEB_DIR=/app/web
USER nonroot
EXPOSE 8080
ENTRYPOINT ["/iasion"]
CMD ["serve"]
