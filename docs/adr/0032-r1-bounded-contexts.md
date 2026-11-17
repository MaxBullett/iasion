# 0032: R1 bounded contexts: Household, Access, Ledger, Classification and Views

- Status: accepted
- Date: 2026-11-17
- Refined by: ADR-0033

## Context

`architecture.md` listed provisional contexts and left their relationships "TBD after
event storming". The R1 event storming (`docs/event-storming/r1.md`) found the
commands, aggregates and invariants of R1. Some invariants decide where things must
live: the Account enforces both its ownership shares and its single opening balance
(ADR-0015, ADR-0019), whether a flow counts in in and out depends on household accounts
and party kinds (ADR-0013, ADR-0023), purpose assignments follow flows across
corrections (ADR-0016, ADR-0017), and voiding a party, an account or a purpose depends
on what refers to it (ADR-0025).

## Decision

1. **R1 touches five contexts:**
   - **Household:** the household, its members, boundary and time zone, and all
     parties with their kinds, including the opening party.
   - **Access:** login identities and who a signed-in request is; the first setup gate.
   - **Ledger:** accounts with owners, ownership shares, institution and implicit
     accounts; opening balances; transactions and flows; checkpoints. Facts, and the
     figures derived from them alone: derived positions with their approximate flag,
     which flows count in in and out, and the allocation of flows to members.
   - **Classification:** the purpose vocabulary and purpose assignments on flows.
   - **Views:** the balance sheet, in and out, and an account's transactions. Reads
     only; it has no aggregates.
2. **Ownership shares live in Ledger,** with the account, not in Household.
3. **Dependencies point one way:** Household is upstream of Ledger and Access; Ledger
   is upstream of Classification; Household, Ledger and Classification are upstream
   of Views. No context depends on a context downstream of it, and there are no
   cycles.
4. **Reference checks go through ports owned upstream.** Where an upstream context
   must know whether a downstream one refers to its data (voiding a party or a member,
   correcting a party's kind away from institution), it defines a port and the
   downstream context implements it.
5. **Contexts refer to each other's things by ID only.**
6. **Cross-context work runs in the application layer:** authenticating a request and
   passing the acting member, computing today, and commands that span contexts (first
   setup, registering a party with its implicit account, voiding a party). Each such
   command is one unit of work.
7. **Each context answers queries on its own data.** Views composes them and never
   reads another context's storage.

## Alternatives considered

- **Ownership shares in Household,** as the provisional table had it: the invariant
  "shares sum to 1 and at most one opening balance" belongs to one aggregate, and
  Ledger needs ownership for every flow rule about household accounts.
- **Parties in Ledger:** members are parties, and Access and the boundary need them
  without the ledger.
- **Purposes and assignments in Ledger:** classification grows later (suggestions,
  plugins, pools, tax categories). Ledger would change with each of them, and an
  assignment's dependence on its flow works from downstream: one whose flow is no
  longer current is not current.
- **Views inside Ledger:** in and out by purpose needs Classification, which is
  downstream of Ledger.
- **Access inside Household:** reaching the app and being a member are two questions
  in two places (ADR-0024).
- **Integration by events between contexts:** whether anything is persisted as an
  event is open (OQ-003). This ADR fixes boundaries and directions, not the mechanism.

## Consequences

- `architecture.md` replaces its provisional table and the "TBD" relationships with
  this context map, including which aggregates and read models live where.
- The persistence ADR must provide one unit of work across Household, Ledger and Access
  for the commands of decision 6, and a check-and-write per aggregate (ADR-0031).
- The kernel primitives (`docs/primitives.md`) are spread over Household (parties,
  boundary) and Ledger (accounts, positions, flows, transactions). Dimensions live in
  Classification.
- Contexts that R1 does not touch (Contracts, Ingestion, Projection, Jurisdiction) stay
  provisional. When they arrive they get their place in the map through an ADR.
- Module and crate layout is for the repo and workspace layout ADR, not this one.
