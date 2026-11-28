#!/bin/sh
# Sets up or updates an Iasion instance on a dedicated Debian 13 VM (ADR-0040,
# decision 17). Run as root from a clone of the repository; running it again applies
# changes to deploy/. See docs/deployment.md.
#
# First run: writes /etc/iasion/iasion.env and /etc/iasion/restic.env and stops, so you
# can fill them in. Next run: generates the database passwords, asks for the tunnel
# token (or takes it from /etc/iasion/tunnel-token, as cloud-init leaves it), installs
# the units and starts the instance.
set -eu

DEPLOY=$(cd "$(dirname "$0")" && pwd)
CONF=/etc/iasion
LIB=/usr/local/lib/iasion
QUADLETS=/etc/containers/systemd
UNITS=/etc/systemd/system
RESTIC_IMAGE=docker.io/restic/restic:0.20.0

say() { printf 'install: %s\n' "$*"; }
fail() { printf 'install: %s\n' "$*" >&2; exit 1; }

[ "$(id -u)" -eq 0 ] || fail "run as root"
if ! grep -q '^VERSION_ID="13"' /etc/os-release 2>/dev/null || ! grep -q '^ID=debian' /etc/os-release; then
    say "warning: written for Debian 13; continuing anyway"
fi

# --- Podman --------------------------------------------------------------------------

if ! command -v podman >/dev/null 2>&1; then
    say "installing Podman"
    apt-get update -q
    DEBIAN_FRONTEND=noninteractive apt-get install -y -q podman
fi

# --- Settings ------------------------------------------------------------------------

mkdir -p "$CONF"
chmod 755 "$CONF"
written=0
if [ ! -f "$CONF/iasion.env" ]; then
    cat >"$CONF/iasion.env" <<'SETTINGS'
# Settings of this Iasion instance (docs/deployment.md). Not secret: the passwords and
# the tunnel token are Podman secrets. Run deploy/install.sh again after a change.

# The image to run and update, e.g. ghcr.io/<owner>/iasion:main
IASION_IMAGE=

# The email allowed to run first setup (ADR-0024).
IASION_SETUP_EMAIL=

# Cloudflare Access (ADR-0037): the team domain, e.g. <team>.cloudflareaccess.com, and
# the AUD tag of the Access application.
IASION_ACCESS_TEAM_DOMAIN=
IASION_ACCESS_AUD=

# The origin the app is served from, e.g. https://iasion.<your domain>
IASION_PUBLIC_ORIGIN=
SETTINGS
    chmod 644 "$CONF/iasion.env"
    written=1
fi
if [ ! -f "$CONF/restic.env" ]; then
    cat >"$CONF/restic.env" <<'SETTINGS'
# Where the backups go (docs/deployment.md, Backups). Secret: readable by root only.
# Keep RESTIC_PASSWORD somewhere off this VM too: without it no backup can be restored.
#
# For a first try, a repository on the VM itself:
#   RESTIC_REPOSITORY=/var/lib/iasion/restic
# For a real instance, one off the VM, e.g. S3-compatible storage:
#   RESTIC_REPOSITORY=s3:https://<endpoint>/<bucket>/iasion
#   AWS_ACCESS_KEY_ID=
#   AWS_SECRET_ACCESS_KEY=
RESTIC_REPOSITORY=
RESTIC_PASSWORD=
SETTINGS
    written=1
fi
chmod 600 "$CONF/restic.env"
if [ "$written" -eq 1 ]; then
    say "wrote the settings templates; fill in $CONF/iasion.env and $CONF/restic.env, then run this again"
    exit 0
fi

value() { sed -n "s/^$1=//p" "$2" | tail -n 1; }
# restic takes anything without a backend prefix as a local path: a repository such
# as `s3.<region>.example.com/<bucket>` would land in the backup container and vanish
# with it, and every backup after it would fail. The env file keeps quotes literally.
check_repository() {
    case "$1" in
        *'"'* | *"'"* | *' '*) fail "RESTIC_REPOSITORY in $CONF/restic.env has quotes or spaces; the file takes values literally" ;;
        /*) ;;
        s3:* | b2:* | sftp:* | rest:* | azure:* | gs:* | swift:* | rclone:*) ;;
        *) fail "RESTIC_REPOSITORY in $CONF/restic.env is neither an absolute path nor a restic backend such as s3:https://<endpoint>/<bucket>/iasion" ;;
    esac
}

require() {
    setting=$(value "$1" "$2")
    [ -n "$setting" ] || fail "$1 is empty in $2"
    case "$setting" in
        *'<'*) fail "$1 in $2 is still a placeholder" ;;
    esac
}
for name in IASION_IMAGE IASION_SETUP_EMAIL IASION_ACCESS_TEAM_DOMAIN IASION_ACCESS_AUD IASION_PUBLIC_ORIGIN; do
    require "$name" "$CONF/iasion.env"
done
for name in RESTIC_REPOSITORY RESTIC_PASSWORD; do
    require "$name" "$CONF/restic.env"
done
check_repository "$(value RESTIC_REPOSITORY "$CONF/restic.env")"
IMAGE=$(value IASION_IMAGE "$CONF/iasion.env")
case "$IMAGE" in
    */*/*) ;;
    *) fail "IASION_IMAGE must name its registry, e.g. ghcr.io/<owner>/iasion:main" ;;
esac

# --- Secrets -------------------------------------------------------------------------

secret_exists() { podman secret exists "$1"; }
secret_create() { printf '%s' "$2" | podman secret create "$1" - >/dev/null; }
password() { od -An -N24 -tx1 /dev/urandom | tr -d ' \n'; }

# A tunnel token is one base64 string that starts with `eyJ`. Anything else, e.g. the
# install command it is shown in, is refused rather than stored.
check_token() {
    case "$1" in
        '' | *'<'*) fail "the tunnel token is empty or still a placeholder" ;;
        *[!A-Za-z0-9+/=_-]*) fail "that is not only the tunnel token: paste just the long string that starts with eyJ, without the command around it" ;;
        eyJ*) ;;
        *) fail "a tunnel token starts with eyJ; paste just the token, without the command around it" ;;
    esac
}

if ! secret_exists iasion-db-owner-password; then
    say "generating the database passwords"
    postgres=$(password)
    owner=$(password)
    app=$(password)
    secret_create iasion-postgres-password "$postgres"
    secret_create iasion-db-owner-password "$owner"
    secret_create iasion-db-app-password "$app"
    secret_create iasion-migrate-database-url "postgres://iasion_owner:$owner@iasion-db:5432/iasion"
    secret_create iasion-database-url "postgres://iasion_app:$app@iasion-db:5432/iasion"
    unset postgres owner app
fi

if ! secret_exists iasion-tunnel-token; then
    if [ -f "$CONF/tunnel-token" ]; then
        # Only the line break at the end is dropped; spaces inside stay, so a pasted
        # command is refused below instead of being glued into one string.
        token=$(tr -d '\r\n' <"$CONF/tunnel-token" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
        rm -f "$CONF/tunnel-token"
    elif [ -t 0 ]; then
        printf 'Cloudflare Tunnel token (docs/deployment.md): '
        stty -echo
        # Echo comes back even if the prompt is left with Ctrl-C.
        trap 'stty echo' EXIT
        trap 'exit 130' INT TERM
        read -r token
        stty echo
        trap - EXIT INT TERM
        printf '\n'
    else
        fail "no tunnel token: put it into $CONF/tunnel-token or run this in a terminal"
    fi
    check_token "$token"
    secret_create iasion-tunnel-token "$token"
    unset token
fi

# --- Files ---------------------------------------------------------------------------

say "installing the units"
mkdir -p "$LIB" "$QUADLETS" "$UNITS/podman-auto-update.timer.d"
install -m 755 "$DEPLOY/postgres/init-roles.sh" "$LIB/init-roles.sh"
install -m 755 "$DEPLOY/bin/iasion-backup" "$LIB/iasion-backup"
for file in "$DEPLOY"/quadlets/*; do
    sed "s|@IASION_IMAGE@|$IMAGE|g" "$file" >"$QUADLETS/$(basename "$file")"
    chmod 644 "$QUADLETS/$(basename "$file")"
done
install -m 644 "$DEPLOY/systemd/iasion-backup.service" "$UNITS/iasion-backup.service"
install -m 644 "$DEPLOY/systemd/iasion-backup.timer" "$UNITS/iasion-backup.timer"
install -m 644 "$DEPLOY/systemd/podman-auto-update-iasion.conf" \
    "$UNITS/podman-auto-update.timer.d/iasion.conf"
systemctl daemon-reload
for unit in iasion-db iasion-migrate iasion-app iasion-cloudflared; do
    systemctl cat "$unit.service" >/dev/null 2>&1 ||
        fail "Podman did not generate $unit.service; see: journalctl -b | grep -i quadlet"
done

# --- Start ---------------------------------------------------------------------------

# Only missing images are pulled. Updating the app's image is podman auto-update's job,
# which rolls back a release that fails to start; pulling here would replace a
# rolled-back image with the failing one.
for image in docker.io/library/postgres:18.7 "$IMAGE" docker.io/cloudflare/cloudflared:2026.11.2 "$RESTIC_IMAGE"; do
    if ! podman image exists "$image"; then
        say "pulling $image"
        podman pull --quiet "$image" >/dev/null
    fi
done

mkdir -p /var/lib/iasion/restic /var/cache/iasion/restic
restic_run() {
    podman run --rm --network host --env-file "$CONF/restic.env" --hostname iasion \
        --volume /var/lib/iasion/restic:/var/lib/iasion/restic \
        --volume /var/cache/iasion/restic:/root/.cache/restic "$RESTIC_IMAGE" "$@"
}
if ! restic_run cat config >/dev/null 2>&1; then
    say "creating the restic repository"
    restic_run init >/dev/null
fi

systemctl enable --now podman-auto-update.timer iasion-backup.timer >/dev/null
say "starting the instance (database, backup, migrations, app, tunnel)"
systemctl restart iasion-app.service
systemctl restart iasion-cloudflared.service

podman exec iasion-app /iasion health >/dev/null || fail "the app is not healthy: journalctl -u iasion-app"
say "done: the app is healthy; see docs/deployment.md for the checks after setup"
