# 0004: AGPL-3.0 license

- Status: accepted
- Date: 2026-09-25
- Refined by: ADR-0042

## Context

The project starts private and is intended to be open-sourced. It is a self-hosted
application that could be run by others as a hosted service.

## Decision

License under the GNU Affero General Public License v3.0.

## Alternatives considered

- MIT/Apache-2.0: maximum adoption, but allows closed hosted forks.
- GPL-3.0: does not cover network use, so hosted forks need not publish changes.

## Consequences

- Modified versions run as a network service must publish their source.
- Compiled-in plugins are very likely derivative works and must be AGPL-compatible.
- The copyright holder can dual-license, as long as outside contributions are covered
  by a CLA or similar (see OQ-007).
