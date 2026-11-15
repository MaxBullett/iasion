# 0024: First setup by a configured email, and login identities map to members

- Status: accepted
- Date: 2026-11-15
- Resolves: OQ-021, except what the acting member is recorded on and whether sign-ins
  are recorded

## Context

S-014 serves a request only if its Cloudflare Access email maps to a member. S-001
creates the household when none exists, which is before any member exists. No command
set the email a member signs in with, nothing said where that mapping lives, and
nothing said that a second household is rejected (`docs/event-storming/r1.md`, H-01,
H-02). `docs/releases/r1.md` said "no user management in the app".

## Decision

1. **Two questions, two places.** Cloudflare Access decides who reaches the app. The
   mapping decides who they are.
2. **Login identity.** A login identity is what a signed-in request is known by; in R1
   that is the email in the verified Access JWT. It maps to exactly one member. A
   member may have several login identities, or none.
3. **The mapping is kept on its own,** not as an attribute of the member.
4. **First setup.** An environment variable names the email that is allowed to run
   first setup. Setup creates the household and its first member and maps that email
   to them.
5. **Afterwards,** members and their login identities are managed in the app.
6. **One household per instance.** A second household is rejected in R1.
7. **The variable is listed in `.env.example`** with a placeholder value. Its name is
   `IASION_SETUP_EMAIL`.

## Alternatives considered

- **The mapping in instance configuration only:** adding a member would mean changing
  the deployment.
- **An email attribute on the member:** one login per member, a member without a login
  becomes a special case, and access details end up in the household model.
- **The first person to sign in becomes the first member:** whoever reaches the
  instance first owns it. It would rely on the Access policy alone.

## Consequences

- While no household exists, only first setup is served, and only for the configured
  email. Afterwards a request whose email is no login identity is denied (S-014).
- S-001 is reworded: setup creates the household with one member; further members are
  added afterwards. S-014 gains criteria for login identities.
- The app holds no passwords or credentials. `docs/releases/r1.md` says so instead of
  "no user management".
- Real email addresses live in the instance's environment and database only, never in
  the repository. `.env.example` uses a placeholder.
- What happens when no login identity maps to any member is open (OQ-040).
- Still open in OQ-021: whether a fact records which member recorded it, and whether
  sign-ins or denied requests are recorded.
- Verifying the Access JWT is part of the authentication enabler, whose ADR is still to
  write.
- The glossary gains login identity.
