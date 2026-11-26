# 0040: Deployment: one image, Podman on a dedicated VM, pulled updates with rollback

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0033, ADR-0039
- Refined by: ADR-0043

## Context

`docs/releases/r1.md` fixes the frame: a Docker image on a self-hosted VM behind
Cloudflare Access, deployed from CI. ADR-0033 requires a backup right before every
migration and runs migrations as a separate step with the owner role. ADR-0039 builds
the image on every pull request and pushes it only from `main`. Setting up and running
the instance should be as easy as possible for one maintainer. The repository
describes a generic setup only; details of a specific instance never enter it
(AGENTS.md).

As of this ADR: Watchtower, the usual pull-based updater for Docker, is archived;
Podman's `auto-update` pulls new images for containers run as systemd units and rolls
back to the previous image when the new one fails to start; Cloudflare Tunnel connects
an origin through outbound connections only.

## Decision

### Image

1. **One image holds the backend and the built frontend,** which the backend serves as
   static files. The app has one origin.
2. **Multi-stage build with a distroless runtime image.** The app runs as a non-root
   user.
3. **The image is published to GitHub's container registry,** public like the code,
   tagged with the commit SHA and `main` (ADR-0039).

### Host

4. **A dedicated Linux VM,** not a system container. Containers inside system
   containers need nesting, which adds setup and fragility.
5. **Podman runs the containers as root inside that VM,** each as a systemd unit
   described by a quadlet file. The VM is the isolation boundary; the app inside its
   container still runs as non-root. Rootless Podman is an optional hardening step in
   the deployment docs. The reference distribution is Debian 13, which ships Podman
   5.4.
6. **The units:** the app, a one-shot migration unit that runs before it, PostgreSQL
   (`postgres:18.7`), `cloudflared`, and a backup timer. They share a private network.
   No port is published on the VM.
7. **Passwords and other secret settings are Podman secrets.** Settings that are not
   secret come from an environment file on the host. Neither is in the repository;
   `.env.example` lists the variables.

### Updates

8. **Pulled, not pushed.** The app unit is marked for `podman auto-update` from the
   registry. A timer checks every 15 minutes. CI never holds credentials for the host.
9. **On an update:** a backup, then `migrate` with the owner role, then the new app. The
   app signals readiness to systemd, so a failed start is detected and Podman rolls
   back to the previous image. The app refuses to start while migrations are pending.

### Migrations under rollback

10. **Every migration keeps the previous release working** ("expand, then contract").
    A rollback runs the previous binary on the migrated schema.
11. **A migration that removes something is a contract migration:**
    - Its file name carries `contract`.
    - It ships in a later release than the expand step it completes.
    - Before it runs, `migrate` takes a backup that is kept until deleted by hand,
      restores it into a scratch database to prove it is usable, and stops if that
      fails.

    ADR-0033 still applies: no migration changes what a stored version says.

### Backups

12. **`pg_dump` every night and before every migration.** The dumps are encrypted and
    copied off the VM with restic, to a target set per instance. Nightly dumps rotate;
    the backups taken before contract migrations do not.
13. **No point-in-time recovery.** At most one day of entries can be lost.
14. **A restore drill is documented** and run by hand at intervals.

### Exposure and operations

15. **`cloudflared` connects outbound only.** The VM accepts no inbound connections;
    Cloudflare Access is in front, and the app verifies every token (ADR-0037).
16. **Logs go to journald.** Health checks run locally on the VM through the app
    binary, never through the tunnel.

### Setup

17. **`deploy/` holds the generic host files:** the quadlet units, a template for the
    instance settings, and an idempotent `deploy/install.sh`. It installs Podman, places
    the units, asks for the secrets and stores them as Podman secrets, and enables the
    timers. Running it again applies changes to `deploy/`.
18. **An optional cloud-init file** runs `install.sh` on a fresh VM's first boot.
19. **`docs/deployment.md`** describes setup, updates, backup and the restore drill, and
    recovery from a lost login identity (ADR-0026), for a generic VM.

## Alternatives considered

- **Docker and Compose on the host:** no maintained pull-based updater, and a
  hand-written pull-and-restart script would have no rollback.
- **A system container instead of a VM:** containers inside it need nesting, and rootless
  Podman there means nested user namespaces.
- **Rootless Podman by default:** UID mapping of volumes and a lingering user add setup,
  while the VM already isolates the app.
- **Deploying by push from CI:** GitHub would hold credentials for the host, and the
  host would need an inbound path.
- **Serving the frontend separately:** a second origin, with CORS and a harder Origin
  check (ADR-0037).
- **A runtime image with a shell:** more to patch and more to attack.
- **Point-in-time recovery with continuous archiving:** more moving parts than a
  household's data needs.
- **Kubernetes or k3s:** orchestration for one container set on one host.
- **Ansible:** one more tool and an inventory to keep out of the repository, for a
  setup that runs rarely. Reconsidered when there is a second host or a second service.
- **Setup steps in the docs only:** rebuilding the VM after a failure would depend on
  following prose correctly.

## Consequences

- `docs/releases/r1.md` links the deployment enabler to this ADR, and `architecture.md`
  lists it.
- "Deployed from CI" means: CI publishes the image, and the host takes it within 15
  minutes.
- Every migration must work with the previous release. Removing anything takes two
  releases.
- The walking skeleton builds `deploy/`, `docs/deployment.md`, the readiness signal and
  the health command.
- Upgrading PostgreSQL to a new major version is decided when it comes up.
