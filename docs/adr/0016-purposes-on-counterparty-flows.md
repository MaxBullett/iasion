# 0016: A purpose attaches to a counterparty-side flow, and flows keep their identity across versions

- Status: accepted
- Date: 2026-11-15
- Refines: ADR-0010
- Resolves: OQ-034, and OQ-029 in part

## Context

S-010 assigns a purpose to a flow. Since ADR-0013 a flow is signed and one-sided, so
even a simple payment has two flows, and it was open which of them carries the
purpose, or whether the transaction does (`docs/event-storming/r1.md`, H-24).

ADR-0010 makes the transaction the versioned unit and a dimension assignment a
versioned fact of its own. What happens to an assignment when its transaction is
corrected or voided was open as well (H-14), and ADR-0015 forbids corrections that
invalidate recorded facts.

## Decision

1. **A purpose attaches to a single flow.**
2. **Only flows that count in in and out can carry one:** a flow on the account of a
   party outside the boundary that is not a system party, in a transaction that has a
   flow on a household account (ADR-0013, decision 8).
3. **Anything else is rejected.** Assigning a purpose to any other flow is rejected
   with a typed error: flows on household accounts, including those of a transfer,
   flows on a system party's account, and flows of a transaction without a flow on a
   household account.
4. **Splits live on the counterparty side.** Example: joint current account -80.00,
   Corner Grocer +50.00 (groceries) and +30.00 (household goods). The purpose of a
   withdrawal fee is on Example Bank's +2.00.
5. **Aggregate flows follow the same rule.**
6. **Members classify their statement line.** The UI shows the line on the household
   account and writes the assignments to the counterparty-side flows.
7. **Flows have identity.** Each flow has a stable ID within its transaction, across
   versions. An assignment references the transaction and the flow.
8. **Assignments follow their flow.** They are dependent facts in the sense of
   ADR-0015 and never block a correction:
   - A correction that changes a flow's amount keeps the flow and its purpose.
   - When a flow is split, one part keeps the ID and the assignment. New parts start
     unclassified.
   - When a flow is removed, or the transaction is voided, its assignments stop being
     current. They are kept in history.

## Alternatives considered

- **Purpose on the transaction:** a split could not be one transaction, so one
  statement line would no longer be one flow on the household account (ADR-0013).
- **Purpose on the household-side flow:** matches what a member sees, but splits do not
  exist on that side, and in and out is read from the other side, so every view would
  have to map across.
- **Purpose on either side:** two places for the same statement, which can disagree.
- **Accept and ignore purposes on other flows:** a member would believe something was
  classified that no view will ever show.
- **Assignments block corrections** (ADR-0015 read strictly): every correction of a
  classified transaction would first need its purposes removed.

## Consequences

- In and out by purpose groups exactly the flows that count, with no mapping between
  sides.
- "Unclassified" means: a flow that can carry a purpose and has none.
- A transaction stores an ID per flow, and a correction states for each flow whether it
  continues an existing one. Flow IDs are part of the persisted format.
- ADR-0010, decision 4, stands: flows are not versioned on their own. They have
  identity, not versions.
- Which flows can carry a purpose is a domain rule. The API exposes it; the frontend
  does not re-implement it (`docs/conventions.md`).
- Not covered, and left in OQ-029: a correction that keeps a flow but makes it unable
  to carry a purpose (e.g. moves it to a household account), removing a purpose
  without replacing it, the confidence of a manual assignment, and whether there is
  any limit to what a correction may change.
- How these flows are attributed to members is open (OQ-035).
- S-010 is rewritten in these terms.
