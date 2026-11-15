# 0013: Flows are signed and one-sided, and every flow is on an account

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0006
- Resolves: OQ-025

## Context

ADR-0006 makes the kernel two-sided: nothing appears or disappears without a
counterparty. `docs/primitives.md` expressed this by defining a flow as a movement of
a quantity from one position to another, and a transaction as a group of flows that
nets to zero per unit.

Event storming for R1 (`docs/event-storming/r1.md`, H-07) showed that this does not
hold together:

- A movement with two ends nets to zero by construction. The transaction's netting
  invariant could never fail, although S-008 expects a version that does not net to
  zero to be rejected.
- The far end of a flow was a position in `docs/primitives.md` but a party in the
  stories ("from ACME GmbH into the account").
- One line on a bank statement is one change to one account. A cash withdrawal with a
  fee is one line of EUR -202.00, but two movements. Import matching and checkpoints
  (R2) compare against statement lines.
- One payment can serve two purposes (a split). As movements, that is two payments.

## Decision

1. **A flow is signed and one-sided.** A flow is a change to the position of one
   account: an account, a unit and a signed amount. A positive amount increases the
   position of that account.
2. **A transaction is a set of flows that nets to zero per unit.** The transaction
   enforces this for every version, and it can fail: a set of flows that does not net
   to zero is rejected. Example, a cash withdrawal of EUR 200.00 with a EUR 2.00 fee:
   Alex's current account -202.00, Alex's cash wallet +200.00, Example Bank +2.00.
3. **Every flow is on an account.** There are no flows directly against a party. Every
   party can own accounts.
4. **Implicit accounts.** Each party outside the boundary, system parties included,
   gets one implicit account, created automatically with the party and owned by it.
   More specific accounts can be registered for such a party later (e.g. a loan
   account).
5. **Members are never a counterparty directly.** Anything a member holds is in an
   account they own, so members get no implicit account. Cash handed to Sam goes into
   Sam's cash wallet. If that is not tracked, it is recorded as Alex's spending, on
   the account of an aggregate party.
6. **A transaction need not touch a household account.** Transactions between outside
   parties are allowed. They affect no household view.
7. **Movements are derived.** A movement, units going from one account to another, is
   a view derived from the flows of a transaction (e.g. for a flow diagram). It is
   never stored.
8. **In and out are read from the other side.** Household in and out are the flows on
   non-household accounts, in transactions that also have a flow on a household
   account. A positive flow on a non-household account is out, a negative one is in.
   Flows on a system party's account stay excluded (ADR-0011). A transaction whose
   flows are all on household accounts is a transfer and adds nothing to in or out.

## Alternatives considered

- **Keep the two-ended movement as the stored unit:** an unbalanced transaction cannot
  even be written down, but the invariant then tests nothing, one statement line maps
  to several stored movements, and fees and splits have to be decomposed at capture
  instead of derived when needed.
- **Signed flows directly against parties, without implicit accounts:** fewer accounts,
  but flows would have two kinds of ends, and "a position is the sum of its flows"
  would hold on the household side only. Claims such as loans need accounts on the
  other side anyway.
- **A new term for the signed line (posting, entry, leg), keeping "flow" for the
  movement:** the new terms are less discoverable, and each suggests that a flow is
  made of two lines. The lines belong to the transaction, which may have any number of
  them. Reading a flow as the change of one stock is also the stock-flow meaning.

## Consequences

- `docs/primitives.md` and `docs/glossary.md` redefine flow and add movement and
  implicit account.
- The netting invariant is a real check, made when a transaction version is
  constructed (`docs/conventions.md`: validated at construction), with a typed error
  and a property test.
- One statement line corresponds to one flow on a household account, which is what R2
  import matching and checkpoints need.
- A split is one flow on the household account and several on the counterparty's
  account. Where purpose assignments attach is open (OQ-034).
- Registering a party creates its implicit account. Creating a household creates the
  opening party and its implicit account. Accounts are no longer owned by members only,
  which narrows OQ-022.
- Implicit accounts have no opening balance and accept flows without one: the opening
  transaction itself has a flow on the opening party's implicit account. Whether the
  opening-balance rules apply to registered accounts of outside parties is open
  (OQ-027).
- The derived position of an outside party's account is the sum of the flows the
  household recorded on it, and says nothing more about that party.
- Earlier ADRs read in these terms:
  - ADR-0009: unchanged. Valid time belongs to the transaction; an aggregate flow is a
    flow of a transaction whose valid time is an interval.
  - ADR-0010: unchanged. The transaction stays the versioned unit.
  - ADR-0011: the opening transaction has two flows, one on the account and the
    opposite one on the opening party's implicit account.
  - ADR-0012: its rule 2 is worded for a transfer with one source, one destination and
    one amount. How in, out and between members are attributed in transactions with
    more flows is open (OQ-035), and ADR-0012 may need a successor.
- Acceptance criteria worded as movements need rewording: S-004 (opening transaction),
  S-005 and S-006 ("a flow from/to a party", "stored as one flow"), S-007 (withdrawal
  with fee as two flows), S-010 (what a purpose is assigned to), S-013 ("flows against
  a system party").
- Members do not enter signed lines. R1 entry forms start from a household account and
  a counterparty, and the flows are composed from that.
- Generators (later) produce expected flows that must net to zero as well. How that is
  expressed is settled when generators are specified.
