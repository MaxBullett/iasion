# Deployment

How to run an Iasion instance, as decided in
[ADR-0040](adr/0040-deployment.md): one image on a dedicated Linux VM, run by Podman as
systemd units, updated by pulling, reachable only through Cloudflare Tunnel behind
Cloudflare Access. This describes a generic setup; the values of a real instance stay
on the instance.

## What runs

| Unit | What it is |
|---|---|
| `iasion-db` | PostgreSQL 18 with its data in the `iasion-db` volume. Ready once it accepts connections. |
| `iasion-migrate` | One shot, before the app: a backup, then `iasion migrate` with the owner role. |
| `iasion-app` | The app: API and frontend on port 8080 of the private network, read-only, without capabilities. Updated by `podman auto-update`. |
| `iasion-cloudflared` | The tunnel. Outbound connections only. |
| `iasion-backup.timer` | The nightly backup. |
| `podman-auto-update.timer` | Checks for a new image every 15 minutes (a drop-in of this repository changes the daily default). |

All containers share the `iasion` network. No port is published on the VM.

On the VM:

- `/etc/iasion/iasion.env` holds the settings that are not secret.
- `/etc/iasion/restic.env` holds where the backups go; root only.
- Podman secrets hold the database passwords and URLs and the tunnel token: `podman secret ls`.
- `/var/lib/iasion/backups` holds the newest dumps; `/var/lib/iasion/restic` holds a local restic repository, if you use one.

## Before you start

### GitHub

1. Apply the repository settings of [ADR-0039](adr/0039-ci-and-secret-scanning.md)
   (decision 17). The required status checks are `secrets`, `rust`, `web`, `e2e`,
   `supply-chain` and `image`. `publish` runs on `main` only and is not one of them.
2. After the first push to `main`, the `publish` job creates the package
   `ghcr.io/<owner>/iasion`. Make it public: **Packages → iasion → Package settings →
   Change visibility → Public**. Otherwise the VM needs `podman login ghcr.io` with a
   token that may read packages.

### Cloudflare

In the Zero Trust dashboard:

1. **Team domain.** Note it, e.g. `<team>.cloudflareaccess.com`: it is
   `IASION_ACCESS_TEAM_DOMAIN`.
2. **A login method**, e.g. one-time PIN by email, or an identity provider.
3. **A tunnel** (Networks → Tunnels), of the cloudflared kind. Copy its token: install.sh
   asks for it. Add a public hostname, e.g. `iasion.<your domain>`, with the service
   `http://iasion-app:8080`.
4. **An Access application** (Access → Applications), self-hosted, for that hostname,
   with a policy that allows the household's emails. Copy its AUD tag: it is
   `IASION_ACCESS_AUD`. `https://` and the hostname are `IASION_PUBLIC_ORIGIN`.

Access decides who reaches the instance; the app still verifies every request's token
([ADR-0037](adr/0037-authentication.md)).

### A VM

- A dedicated VM with Debian 13, 2 GB of memory and 20 GB of disk or more. Not a system
  container: containers inside one need nesting (ADR-0040, decision 4).
- Outbound access to the internet. No inbound access is needed.

### A backup target

A [restic](https://restic.net) repository off the VM, e.g. S3-compatible storage, and a
password for it. Keep the password somewhere off the VM too: without it no backup can
be restored. For a first try, a repository on the VM itself works.

## Setup

As root on the VM:

```sh
apt-get update && apt-get install -y git
git clone <repository URL> /opt/iasion
sh /opt/iasion/deploy/install.sh
```

The first run writes `/etc/iasion/iasion.env` and `/etc/iasion/restic.env` and stops.
Fill them in:

- `IASION_IMAGE`: `ghcr.io/<owner>/iasion:main`, or a commit's tag to pin a release.
- `IASION_SETUP_EMAIL`, `IASION_ACCESS_TEAM_DOMAIN`, `IASION_ACCESS_AUD`,
  `IASION_PUBLIC_ORIGIN`: see `.env.example` and the steps above.
- `RESTIC_REPOSITORY`, `RESTIC_PASSWORD`, and the backend's credentials if it needs
  any.

Then run it again:

```sh
sh /opt/iasion/deploy/install.sh
```

It installs Podman if needed, generates the database passwords, asks for the tunnel
token, installs the units, creates the restic repository, and starts the instance. It
ends with `install: done: the app is healthy`.

### With cloud-init

`deploy/cloud-init.yaml` does the same on a fresh VM's first boot, without prompts.
Copy it outside the repository, replace every `<...>`, and give the copy to the VM as
its user data. The copy holds secrets: never commit it. Follow the setup in
`/var/log/cloud-init-output.log`.

### Checks after setup

```sh
podman ps                                   # iasion-db (healthy), iasion-app, iasion-cloudflared
podman exec iasion-app /iasion health       # health: ok
sleep 120; podman ps --filter name=iasion-app   # still Up, and (healthy): Podman's own check passes
systemctl show iasion-app -p NRestarts      # 0, and it stays there
systemctl list-timers iasion-backup.timer podman-auto-update.timer
journalctl -u iasion-cloudflared -n 20      # the tunnel's connections
```

Then open the public hostname: Access asks you to sign in, and the page shows the setup
email as signed in.

### Changing settings or files

Edit `/etc/iasion/iasion.env`, or update the clone (`git -C /opt/iasion pull`), and run
`install.sh` again. It keeps the secrets, reinstalls the units and restarts the
instance with the images it already has.

To replace a secret, remove it and run `install.sh` again. For example, for a new
tunnel token: `podman secret rm iasion-tunnel-token`.

## Updates

`publish` pushes a new image for every commit on `main`. Within 15 minutes,
`podman auto-update` pulls it and restarts the app. Starting the app first runs
`iasion-migrate`: a backup, then the migrations. The app reports itself ready only once
it listens.

If the new app does not start, Podman rolls back to the previous image and starts it
again. The previous release runs on the migrated schema, which every migration must
allow (ADR-0040, decision 10), and its migrate step accepts the newer migrations. The
broken release is tried again at every check, each time with a backup, until `main` has
a fix.

- Watch: `journalctl -u podman-auto-update -u iasion-migrate -u iasion-app`.
- Update now: `podman auto-update`. See what would update: `podman auto-update --dry-run`.
- Pin a release: set `IASION_IMAGE` to `ghcr.io/<owner>/iasion:<commit SHA>` and run
  `install.sh`.
- If the app is stopped, `podman auto-update` leaves it alone. Pull and start it by
  hand: `podman pull "$IMAGE" && systemctl start iasion-app`, with the image of
  `iasion.env`.

`install.sh` never pulls an image it already has, so running it again never replaces
an image that auto-update rolled back.

## Backups

`iasion-backup` dumps the database with `pg_dump` through the database container and
copies the dump to the restic repository, encrypted (ADR-0040, decisions 12 and 13).

| When | Tag | Kept in restic |
|---|---|---|
| Every night, around 03:30 | `nightly` | 7 daily, 4 weekly, 12 monthly |
| Before every migration | `pre-migrate` | 30 days |

The dumps also stay in `/var/lib/iasion/backups` for three days. There is no
point-in-time recovery: at most a day of entries can be lost (decision 13).

Take one now: `systemctl start iasion-backup`.

List the snapshots:

```sh
podman run --rm --env-file /etc/iasion/restic.env \
  -v /var/lib/iasion/restic:/var/lib/iasion/restic \
  docker.io/restic/restic:0.20.0 snapshots
```

A migration that removes something (a contract migration, decision 11) needs a backup
that is kept and checked first. `iasion migrate` does not take that yet, so
`cargo xtask migrations` refuses such migrations until it does.

## Restore drill

Run it every few months, and after changing the backup target (decision 14). It
restores the newest nightly backup into a scratch database next to the live one, and
leaves the live one alone. As root:

```sh
mkdir -p /root/restore
podman run --rm --env-file /etc/iasion/restic.env \
  -v /var/lib/iasion/restic:/var/lib/iasion/restic -v /root/restore:/restore \
  docker.io/restic/restic:0.20.0 restore latest --tag nightly --target /restore
dump=$(ls -t /root/restore/backups/*.dump | head -n 1)
podman exec iasion-db createdb -U postgres restore_drill
podman exec -i iasion-db pg_restore -U postgres -d restore_drill --no-owner < "$dump"
podman exec iasion-db psql -U postgres -d restore_drill -c 'SELECT max(version) FROM _sqlx_migrations'
podman exec iasion-db dropdb -U postgres restore_drill
rm -rf /root/restore
```

The drill passes when `pg_restore` reports no errors and the migration version is the
one the instance runs.

## Restoring for real

This replaces the live database with a backup, for example to undo a failed release
(ADR-0033, decision 16). Entries made after the backup are lost. As root:

1. Restore the dump you need into `/root/restore`, as in the drill. List the snapshots
   to pick one other than the latest: `restore <snapshot ID>`.
2. Stop the app and the tunnel:
   `systemctl stop iasion-cloudflared iasion-app`
3. Replace the database. `pg_dump` does not carry the database's own grants, so the two
   from `deploy/postgres/init-roles.sh` are set again:

   ```sh
   podman exec iasion-db dropdb -U postgres iasion
   podman exec iasion-db createdb -U postgres --owner iasion_owner iasion
   podman exec iasion-db psql -U postgres -c 'REVOKE ALL ON DATABASE iasion FROM PUBLIC; GRANT CONNECT ON DATABASE iasion TO iasion_app'
   podman exec -i iasion-db pg_restore -U postgres -d iasion --exit-on-error < "$dump"
   ```

4. If the backup is older than the running release's migrations, pin the release that
   wrote it (see Updates) before starting.
5. Start again: `systemctl start iasion-app iasion-cloudflared`, then
   `podman exec iasion-app /iasion health`.

## A lost login identity

A household always has at least one login identity; recovering access when the last one
is lost is an operator task ([ADR-0026](adr/0026-last-login-identity-and-recovery.md)). The command
for it comes with the login identities themselves (S-014), and its steps go here then.

## Hardening

- Keep Debian's security updates on, e.g. with `unattended-upgrades`.
- Rootless Podman is possible but not set up by `install.sh`: it needs a lingering user
  and UID mapping for the volume (ADR-0040, decision 5).
- Nothing listens on the VM's interfaces except SSH. Keep it that way: the tunnel needs
  no inbound port.
