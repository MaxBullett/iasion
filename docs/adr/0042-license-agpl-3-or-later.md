# 0042: The license is AGPL-3.0-or-later

- Status: accepted
- Date: 2026-11-26
- Refines: ADR-0004

## Context

ADR-0004 licenses Iasion under the GNU Affero General Public License v3.0, without
saying whether later versions of that license apply. Cargo manifests, the web
package and `cargo-deny` (ADR-0039) need an exact SPDX identifier: `AGPL-3.0-only` or
`AGPL-3.0-or-later`. The license text is the same for both; they differ in whether a
recipient may also use the code under a later version the Free Software Foundation
publishes. How contributors' rights are handled is still open (OQ-007).

## Decision

1. **The license is AGPL-3.0-or-later:** version 3 of the GNU Affero General Public
   License, or, at the recipient's option, any later version.
2. **The SPDX identifier `AGPL-3.0-or-later`** is stated in the Cargo workspace
   (`license`, inherited by every crate) and in the web package's `package.json`.
3. **`LICENSE` stays the verbatim license text.** The README states the license with the
   "or any later version" notice.
4. **No per-file license headers.** The manifests and the README carry the license.

## Alternatives considered

- **AGPL-3.0-only:** moving to a later version would need every copyright holder's
  agreement, which gets hard once others contribute (OQ-007).
- **Per-file headers:** noise in every file, for a repository whose license is stated
  once and unambiguously.

## Consequences

- Recipients may use the code under a later AGPL version once one exists, including one
  the maintainer would not have chosen.
- The allow list of `cargo-deny` and of the web license check is judged against
  AGPL-3.0-or-later.
- OQ-007 stays open; it decides how outside contributions keep the licensing options
  open.
