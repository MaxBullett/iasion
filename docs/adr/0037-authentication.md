# 0037: Authentication: verify the Cloudflare Access JWT, map its email to a member

- Status: accepted
- Date: 2026-11-18
- Refines: ADR-0024, ADR-0036
- Refined by: ADR-0038

## Context

Cloudflare Access decides who reaches the app; the mapping of login identities decides
who they are (ADR-0024). S-014 rejects a request without a valid Access JWT, even if an
email header is present. The authentication enabler in `docs/releases/r1.md` had no
ADR. Sign-ins and denied requests are operational logs only (ADR-0028).

Cloudflare's documentation on validating the Access JWT says: the token arrives in the
`Cf-Access-Jwt-Assertion` header, and the `CF_Authorization` cookie is not guaranteed
to be passed; tokens are signed with RS256; the public keys are fetched from the team's
`/cdn-cgi/access/certs` endpoint and matched by `kid` in `public_certs`; `aud`, `iss`,
`exp` and `nbf` must be validated, because a valid signature alone does not make a
token valid for the application; keys rotate every six weeks and the previous keys stay
valid for seven days.

## Decision

### Verifying a request

1. **Only the `Cf-Access-Jwt-Assertion` header is read,** never the cookie and never an
   email header.
2. **The token is verified with RS256 only,** against a key from the team's certs
   endpoint whose `kid` matches. `aud` must contain the application's AUD tag, `iss`
   must be the team domain, and `exp` and `nbf` must hold, with a tolerance of 60
   seconds. All four claims are required.
3. **The team domain and the AUD tag come from the environment:**
   `IASION_ACCESS_TEAM_DOMAIN` and `IASION_ACCESS_AUD`, listed in `.env.example` with
   placeholders. The real values are deployment details and never enter the
   repository.
4. **Library:** `jsonwebtoken`. Its defaults do not fit and are set explicitly: the
   accepted algorithms (default HS256), `nbf` validation (off by default) and the
   required claims (default `exp` only).

### Keys

5. **Keys are fetched at startup and kept in memory.** They are fetched again when a
   token names an unknown `kid` (at most once a minute) and every hour.
6. **Fail closed.** If no valid key is available, a request is refused with 503 and
   nothing is served.

### From token to member

7. **The `email` claim is looked up as a login identity** (ADR-0024). A token without
   an email claim is denied, whatever kind of token it is; R1 has no machine clients.
8. **Emails are stored trimmed and lowercased and compared that way,** including
   `IASION_SETUP_EMAIL`.
9. **A tower layer in `api`** verifies the token, resolves the login identity through
   Access, and passes the acting member into `app` (ADR-0034, ADR-0035). While no
   household exists, only first setup is served, and only for the setup email
   (ADR-0024).

### Cross-site requests

10. **Commands accept only `application/json`,** and the API sends no CORS headers that
    would allow another origin.
11. **Every request other than GET must carry an `Origin` header** equal to the app's
    own origin, set as `IASION_PUBLIC_ORIGIN`. Otherwise it is rejected. The browser
    sends the `CF_Authorization` cookie with cross-site requests too, and Access then
    adds the header, so without these checks another site could issue commands in a
    member's name.

### Development and tests

12. **There is no switch that disables authentication.** For local development, a
    dev-only tool serves a local key set and issues tokens; the verification code is
    the same, only its configuration points elsewhere.
13. **Acceptance tests need no token:** they call `app` with the acting member directly
    (ADR-0034). API tests use a test key pair.

### Logs

14. **Logs contain no email and no token.** A denied request is logged with its reason
    and a request ID (ADR-0028).

## Alternatives considered

- **Trusting the email header Access also sets:** anyone who reaches the origin
  without passing Access could set it (S-014).
- **Reading the cookie as a fallback:** the documentation says it is not guaranteed to
  arrive, and the header carries the same token.
- **A hard-coded public key:** keys rotate every six weeks.
- **Serving without a valid key when the certs endpoint is down:** requests would be
  served unverified.
- **Comparing emails exactly as given:** "Alex@" and "alex@" would be two login
  identities for one mailbox in practice.
- **A flag that disables authentication in development:** it can reach production.
- **Relying on the cookie's SameSite attribute against cross-site requests:** it is
  configured in Cloudflare, outside the repository, and can change unnoticed.
- **Logging emails of denied requests:** every log file would hold personal data.

## Consequences

- `.env.example` gains three variables. `docs/releases/r1.md` links the authentication
  enabler to this ADR, and `architecture.md` lists it.
- Emails that differ only in case are one login identity.
- The dev environment ADR provides the local key set and token tool.
- If a later release adds machine clients through Access service tokens, how they are
  identified is decided then.
