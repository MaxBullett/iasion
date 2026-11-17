# Architecture decision records

Format: [MADR](https://adr.github.io/madr/), simplified. Copy
[0000-template.md](0000-template.md) for new records.

ADRs are immutable once accepted. To change a decision, write a new ADR that
supersedes the old one and update the old one's status.

| ADR | Title | Status |
|---|---|---|
| [0001](0001-spec-driven-test-first.md) | Spec-driven, test-first development | Accepted |
| [0002](0002-rust-backend.md) | Rust for the backend | Accepted |
| [0003](0003-vue-frontend.md) | Vue 3 and TypeScript for the frontend | Accepted |
| [0004](0004-agpl-license.md) | AGPL-3.0 license | Accepted |
| [0005](0005-plugins-as-compiled-crates.md) | Plugins as compiled-in crates, independent of the frontend | Accepted |
| [0006](0006-stock-flow-kernel.md) | Stock-flow kernel with two-sided flows | Accepted |
| [0007](0007-onboarding-present-first.md) | Onboarding present first, with explicit coverage and gaps | Accepted |
| [0008](0008-responsive-web-no-native.md) | Responsive web app, installable as a PWA, no native or WASM frontend | Accepted |
| [0009](0009-valid-time-intervals-for-aggregate-flows.md) | Valid-time intervals for aggregate flows | Accepted |
| [0010](0010-corrections-as-versions.md) | Corrections as versions of a fact | Accepted |
| [0011](0011-opening-balance-as-transaction.md) | Opening balance as a transaction against an opening party | Accepted |
| [0012](0012-between-members-line.md) | Transfers between members appear on a separate line in member views | Accepted |
| [0013](0013-signed-flows-on-accounts.md) | Flows are signed and one-sided, and every flow is on an account | Accepted |
| [0014](0014-end-of-day-positions-and-occurred-at.md) | Positions are end-of-day, and a transaction may carry an occurred-at time | Accepted |
| [0015](0015-life-of-an-opening-balance.md) | An opening balance is one fact, comes first, and corrections never invalidate independent facts | Accepted |
| [0016](0016-purposes-on-counterparty-flows.md) | A purpose attaches to a counterparty-side flow, and flows keep their identity across versions | Accepted |
| [0017](0017-ending-assignments-and-limits-of-corrections.md) | Purpose assignments end without blocking, and a correction may change anything but identity | Accepted |
| [0018](0018-member-attribution-of-in-and-out.md) | In and out are attributed to whose money paid and whose account received | Accepted |
| [0019](0019-exact-shares-and-whole-minor-units.md) | Ownership shares are exact fractions, and attribution allocates whole minor units | Accepted |
| [0020](0020-reconciliation-line-per-system-party.md) | The reconciliation has one line per system party | Accepted |
| [0021](0021-figures-carry-flags.md) | Figures carry flags, and a total takes the union | Accepted |
| [0022](0022-in-out-and-income-spending.md) | In and out are kernel terms; income and spending are view terms, netted per purpose | Accepted |
| [0023](0023-household-accounts-owned-by-members.md) | In R1 a household account is owned by members only, and the household is not a party | Accepted |
| [0024](0024-first-setup-and-login-identities.md) | First setup by a configured email, and login identities map to members | Accepted |
| [0025](0025-corrections-to-household-data.md) | Parties, accounts, ownership shares and purposes are corrected as versions | Accepted |
| [0026](0026-last-login-identity-and-recovery.md) | The last login identity cannot be removed, and recovery is an operator task | Accepted |
| [0027](0027-institution-is-a-party-of-kind-institution.md) | An account's institution is a party of kind institution | Accepted |
| [0028](0028-fact-versions-record-the-member.md) | Every fact version records the member who recorded it; sign-ins are not facts | Accepted |
| [0029](0029-comparing-names.md) | Names are stored trimmed and compared ignoring case | Accepted |
| [0030](0030-limits-on-recorded-values.md) | Limits on recorded values: no future dates, no zero flows, one checkpoint per date | Accepted |
| [0031](0031-corrections-name-the-version-they-replace.md) | A correction names the version it replaces | Accepted |
| [0032](0032-r1-bounded-contexts.md) | R1 bounded contexts: Household, Access, Ledger, Classification and Views | Accepted |
| [0033](0033-persistence-postgresql-versioned-facts.md) | Persistence: PostgreSQL only, a schema per context, versioned facts in append-only tables | Accepted |
| [0034](0034-workspace-layout.md) | Workspace layout: one crate per context, persistence behind a feature | Accepted |
