# Story map

Status: backbone, A1 tasks, first A2 tasks, R1 stories (draft, under review).
Tasks for A3 to A5 and stories beyond R1 follow.

Left to right is the user's flow, not build order. Build order comes from release slices.

## Backbone

| ID | Activity | Description |
|---|---|---|
| A1 | Capture & maintain | Record parties, accounts, positions, flows and contracts, and keep them current as life changes. Manual first, import later. |
| A2 | Understand the present | Balance sheet, net worth, cash flows, where money comes from and goes, per member and per pool. |
| A3 | Explore the future | Projections and scenarios, shown as distributions. |
| A4 | Decide | Compare strategies and commit to a plan. |
| A5 | Monitor | Plan vs. actual, forecast error, drift, attribution. |

## Tasks

Task IDs are `A<activity>.<n>`, numbered in order of addition. An ID is never reused or
renumbered, even if a task is renamed, moved or dropped; new tasks get the next free
number for their activity. The release column shows the first release that touches a
task; later releases may extend it.

### A1: Capture & maintain

Onboarding follows [ADR-0007](adr/0007-onboarding-present-first.md).

| ID | Task | Description | Release |
|---|---|---|---|
| A1.1 | Set up household | Members and boundary. | R1 |
| A1.2 | Register parties | Including aggregate parties. | R1 |
| A1.3 | Register accounts | Owners, ownership shares, optional institution. | R1 |
| A1.4 | Establish starting point | Opening balance per account, i.e. its opening positions. | R1 |
| A1.5 | Record flows | Income, spending, transfers between own accounts. | R1 |
| A1.6 | Record checkpoints | Observed balances, compared against derived positions. | R1 |
| A1.7 | Classify | Assign dimensions, confirm suggestions. | R1 (manual purpose only) |
| A1.8 | Correct the past | Corrections are new facts with the correct valid time. | R1 (correct or void a flow; correct household data) |
| A1.9 | Register contracts | Each contract creates a contractual generator. | later |
| A1.10 | Record valuations | | later |
| A1.11 | Attach documents | | later |
| A1.12 | Maintain contracts over time | Rate resets, prepayments, end dates. | later |
| A1.13 | Import & confirm | | R2 |

### A2: Understand the present

| ID | Task | Description | Release |
|---|---|---|---|
| A2.1 | View balance sheet | Positions and net worth as of a date, per member and household. | R1 |
| A2.2 | View in and out | Inflows and outflows by purpose over a period, per member and household. | R1 |

### A3 to A5

_TBD_

## Release slices

| Release | Theme | Definition |
|---|---|---|
| R1 | Manual ledger | [releases/r1.md](releases/r1.md) |
| R2 | Import and reconciliation (provisional) | Import, deduplication, transfer matching, resolving checkpoint differences |

## Stories

Format: `S-###`, activity, task ID, story, acceptance criteria (Given/When/Then).
Notes marked **OQ** depend on an open question and are provisional until it is resolved.
Cross-cutting stories that belong to no activity are marked with activity `-`.

### S-001: Set up the household

- Activity: A1. Task: A1.1 Set up household. Release: R1.
- As a household member, I want to create our household with its members, so that
  everything recorded later is attributed to the right people.

Acceptance criteria:

- Given no household exists and a request from the email named for first setup, when I
  run first setup with member "Alex", then the household exists, Alex is a member
  party inside the boundary, and that email is mapped to Alex (ADR-0024).
- Given no household exists and a request from any other email, then first setup is
  refused.
- Given a household exists, when first setup is run again, then it is rejected: an
  instance holds one household (ADR-0024).
- Given a household exists, when I add a member, then the new member is inside the
  boundary and existing facts are unchanged.
- Given a member, when I look at their party kind, then it is natural person.

### S-002: Register a counterparty

- Activity: A1. Task: A1.2 Register parties. Release: R1.
- As a household member, I want to register the parties money comes from and goes to,
  so that every flow has a counterparty.

Acceptance criteria:

- Given a household, when I register "ACME GmbH" with kind legal entity, then it exists
  as a party outside the boundary.
- Given a household, when I register an aggregate party "Grocery merchants", then it can
  be used as a counterparty like any other party.
- Given a registered party, then it has exactly one implicit account, owned by it
  (ADR-0013).
- Given a party name that already exists, when I register it again, then I am warned
  and can still proceed (two different parties may share a name).

### S-003: Register an account

- Activity: A1. Task: A1.3 Register accounts. Release: R1.
- As a household member, I want to register an account with its owners, so that
  positions and flows are attributed correctly.

Acceptance criteria:

- Given members Alex and Sam, when I register a joint current account at a bank with
  ownership shares 50% and 50%, then the account exists with both owners.
- Given ownership shares that do not sum to 100%, when I register the account, then it
  is rejected with a typed error naming the actual sum.
- Given members Alex and Sam, when I register an account with ownership shares 1/3 and
  2/3, then it is accepted: shares are exact fractions that sum to exactly 1
  (ADR-0019).
- Given member Alex, when I register a cash wallet with no institution and Alex as sole
  owner, then the account exists.
- Given member Alex and party "ACME GmbH", when I register an account owned by both,
  then it is rejected with a typed error: the owners of an account are either all
  members or all outside the boundary (ADR-0023).
- Given any account in R1, when I look at its units, then it holds only EUR.

### S-004: Record an opening balance

- Activity: A1. Task: A1.4 Establish starting point. Release: R1.
- As a household member, I want to state an account's balance at a date, so that I can
  start recording without its full history.

Acceptance criteria:

- Given an account without an opening balance, when I record an opening balance of
  EUR 2,450.00 for the end of 2026-09-30, then the derived position at 2026-09-30 is
  EUR 2,450.00.
- Given an account with an opening balance, when I record a second one, then it is
  rejected. (Changing it is a correction, see S-008.)
- Given a new account opened on 2026-10-15, when I record its starting point as "new
  account", then an opening balance of EUR 0.00 is recorded for the end of 2026-10-14
  (ADR-0015).
- Given an account without an opening balance, when I view the balance sheet, then the
  account is shown as having no starting point, not as zero.
- Given an opening balance of EUR 2,450.00, then it is stored as a checkpoint and a
  transaction with two flows, EUR +2,450.00 on the account and EUR -2,450.00 on the
  opening party's implicit account, and the derived position equals the sum of flows.
- Given an opening balance, when I view in and out for its period, then it does not
  appear as income.
- Given a household is created, then exactly one opening party exists for it, with one
  implicit account.

### S-005: Record income

- Activity: A1. Task: A1.5 Record flows. Release: R1.
- As a household member, I want to record money coming in, so that I can see where
  money comes from.

Acceptance criteria:

- Given an account with an opening balance at 2026-09-30 and employer party "ACME GmbH",
  when I record income of EUR 3,800.00 from ACME GmbH into the account at 2026-10-28,
  then one transaction is stored with two flows, EUR +3,800.00 on the account and
  EUR -3,800.00 on ACME GmbH's implicit account, and the derived position at
  2026-10-28 has increased by EUR 3,800.00.
- Given any recorded transaction, then its flows net to zero per unit and the flow on
  the counterparty's account is visible.
- Given flows that do not net to zero per unit, when the transaction is recorded, then
  it is rejected with a typed error (ADR-0013).
- Given any recorded transaction, then it carries a valid time (the date entered) and a
  recorded time (when it was saved).
- Given an account with an opening balance at 2026-09-30, when I record a flow on it
  dated 2026-09-30 or earlier, then it is rejected with a typed error. (The opening
  balance is the end-of-day balance, ADR-0014. R1 has no backfill feature; moving the
  opening balance to an earlier date is a correction, see S-008.)
- Given an account without an opening balance, when I record a flow on it, then it is
  rejected with a typed error (ADR-0015).

### S-006: Record spending

- Activity: A1. Task: A1.5 Record flows. Release: R1.
- As a household member, I want to record money leaving, individually or as an
  aggregate, so that capture stays cheap.

Acceptance criteria:

- Given an account and party "Corner Grocer", when I record spending of EUR 54.20 at
  Corner Grocer at 2026-10-14, then one transaction is stored with two flows,
  EUR -54.20 on the account and EUR +54.20 on Corner Grocer's implicit account, and
  the derived position decreases by EUR 54.20 from that date.
- Given an account and aggregate party "Grocery merchants", when I record an aggregate
  flow of EUR 640.00 for 2026-10-01 to 2026-10-31, then it is stored as one
  transaction with that valid-time interval and two flows, EUR -640.00 on the account
  and EUR +640.00 on the implicit account of Grocery merchants.
- Given that aggregate flow, when I view the derived position as of 2026-10-15, then
  it excludes the EUR 640.00 and is marked approximate.
- Given that aggregate flow, when I view the derived position as of 2026-10-01, the
  first day of the interval, then it is marked approximate as well (ADR-0014).
- Given that aggregate flow, when I view the derived position as of 2026-10-31 or
  later, then it includes the full EUR 640.00 and is not marked approximate.
- Given an interval whose end is before its start, when I record the flow, then it is
  rejected.

### S-007: Record a transfer between own accounts

- Activity: A1. Task: A1.5 Record flows. Release: R1.
- As a household member, I want to record money moving between household accounts, so
  that it is not counted as income or spending.

Acceptance criteria:

- Given Alex's current account and Alex's cash wallet, when I record a cash withdrawal of
  EUR 200.00 at 2026-10-10, then it is one transaction with two flows: EUR -200.00 on
  the current account and EUR +200.00 on the wallet.
- Given the same withdrawal with a EUR 2.00 fee, when I record it, then it is one
  transaction with three flows: EUR -202.00 on the current account, EUR +200.00 on the
  wallet and EUR +2.00 on Example Bank's implicit account.
- Given that withdrawal with a fee, when I view household in and out, then only the
  EUR 2.00 on Example Bank's account appears, as out; the flows on household accounts
  do not.
- Given a transfer of EUR 1,000.00 from Alex's account to the joint 50/50 account, when
  I view per member, then Alex shows EUR -500.00 and Sam EUR +500.00 under "between
  members", and neither shows it as in or out (ADR-0012).
- Given a transfer into a household account without an opening balance, when I record
  it, then it is rejected with a typed error (ADR-0015).

### S-008: Correct or void a flow

- Activity: A1. Task: A1.8 Correct the past. Release: R1.
- As a household member, I want to fix a flow I entered wrongly, so that mistakes do not
  stay in the figures.

Acceptance criteria:

- Given a recorded flow of EUR 54.20, when I correct the amount to EUR 45.20, then views
  use EUR 45.20 and the original fact is still stored with its recorded time.
- Given a recorded flow, when I void it, then it no longer affects any view, and the
  original fact is still stored.
- Given a cash withdrawal with a fee, when I correct the fee, then a new version of the
  whole transaction is stored and it still nets to zero; a version that does not net to
  zero is rejected.
- Given a corrected flow, when I view figures as known on a date before the correction
  was recorded, then the original amount is used.
- Given a correction, then no reversing flow appears in any view.
- Given an opening balance, when I correct its amount, then a new version of the
  opening balance is stored, with its checkpoint and its transaction written together
  (ADR-0015).
- Given an opening balance at 2026-09-30, when I correct its date to an earlier date,
  then it is accepted.
- Given an opening balance at 2026-09-30 and a flow on the account at 2026-10-14, when
  I correct the opening date to 2026-10-14 or later, then it is rejected with a typed
  error naming the flow. The same holds for a checkpoint on the account.
- Given an opening balance on an account with other flows or checkpoints, when I void
  it, then it is rejected.
- Given an opening balance on an account with no other facts, when I void it, then a
  new opening balance can be recorded for that account.

### S-009: Define purposes

- Activity: A1. Task: A1.7 Classify. Release: R1.
- As a household member, I want to define our own purposes (e.g. groceries, rent,
  transport), so that spending is grouped the way we think about it.

Acceptance criteria:

- Given a household, when I create the purpose "Groceries", then it can be assigned to
  flows.
- Given a purpose with assigned flows, when I rename it, then the assignments keep
  pointing to it.
- Given a purpose name that already exists, when I create it again, then it is rejected.

### S-010: Assign a purpose to a flow

- Activity: A1. Task: A1.7 Classify. Release: R1.
- As a household member, I want to assign a purpose to a flow, so that I can see where
  money goes.

Acceptance criteria:

- Given spending of EUR 54.20 at Corner Grocer and the purpose "Groceries", when I
  assign it, then the assignment is on the flow of EUR +54.20 on Corner Grocer's
  implicit account and has provenance: assigned by me, confirmed, with confidence
  certain (ADR-0017).
- Given a payment of EUR 80.00 from the joint current account with two flows on Corner
  Grocer's implicit account, EUR +50.00 and EUR +30.00, when I assign "Groceries" to
  the first and "Household goods" to the second, then each flow has its own purpose.
- Given a cash withdrawal with a EUR 2.00 fee (S-007), when I assign the purpose "Bank
  fees", then it is on the flow of EUR +2.00 on Example Bank's implicit account.
- Given the aggregate flow of EUR 640.00 (S-006), when I assign "Groceries", then the
  assignment is on the flow on the implicit account of Grocery merchants.
- Given a flow on a household account, a flow on a system party's account, or a flow
  of a transaction that has no flow on a household account, when I assign a purpose
  to it, then it is rejected with a typed error (ADR-0016).
- Given a flow that can carry a purpose and has none, when I view in and out, then it
  appears under "Unclassified", never dropped.
- Given a flow with a purpose, when I change the purpose, then the previous assignment
  is kept with its recorded time.
- Given a flow with a purpose, when I correct its amount (S-008), then the flow keeps
  its purpose.
- Given a flow with a purpose, when a correction splits it in two, then one part keeps
  the purpose and the other appears under "Unclassified".
- Given a flow with a purpose, when a correction removes the flow or the transaction
  is voided, then the assignment is no longer current and is still stored.
- Given a flow with a purpose, when a correction moves it to a household account, then
  I am warned before saving, and afterwards the assignment is no longer current and
  is still stored (ADR-0017).
- Given a flow with a purpose, when I remove the purpose, then the flow appears under
  "Unclassified" and the previous assignment is kept with its recorded time
  (ADR-0017).

### S-011: Record a checkpoint

- Activity: A1. Task: A1.6 Record checkpoints. Release: R1.
- As a household member, I want to enter a balance I can see on a statement or count in
  my wallet, so that I know whether my records are complete.

Acceptance criteria:

- Given an account with a derived position of EUR 310.00 at 2026-10-31, when I record a
  checkpoint of EUR 260.00 at 2026-10-31, then the difference of EUR -50.00 is shown.
- Given a checkpoint with a difference, then no flow is booked automatically.
- Given a checkpoint with a difference, when I record the missing flows, then the
  difference shown updates accordingly.
- Given a checkpoint dated before the account's opening balance, when I record it, then
  it is rejected.
- Given an account without an opening balance, when I record a checkpoint, then it is
  rejected (ADR-0015).
- Given a checkpoint dated on or after the first day and before the last day of an
  aggregate flow's interval on that account, then the difference is shown as
  approximate.
- Given an account with several checkpoints, when I view the account's checkpoints,
  then each is listed with its date, its amount and its current difference.
- Given a checkpoint with a difference followed by a later checkpoint without one,
  then the list still shows the earlier difference at its own date.

### S-012: View the balance sheet

- Activity: A2. Task: A2.1 View balance sheet. Release: R1.
- As a household member, I want to see what we own at a date, so that I know where we
  stand.

Acceptance criteria:

- Given accounts with opening balances and flows, when I view the balance sheet as of
  2026-10-31, then each account shows its derived position as of that date.
- Given a joint account with shares 50/50 and a position of EUR 1,000.00, when I view
  the balance sheet per member, then each member is attributed EUR 500.00.
- Given all accounts, then household net worth equals the sum of all positions of
  household accounts, and equals the sum of the per-member figures.
- Given an account whose latest checkpoint on or before the as-of date has a
  difference, then the balance sheet shows that the account has an unexplained
  difference. Later checkpoints are not considered.
- Given an as-of date before an account's opening balance, then that account is shown
  as unknown, not as zero, and net worth is the total of the other accounts, flagged
  partial and naming that account as not tracked (ADR-0021).
- Given an as-of date on or after the first day and before the last day of an
  aggregate flow's interval, then the affected account and the totals it feeds into
  are marked approximate.

### S-013: View in and out

- Activity: A2. Task: A2.2 View in and out. Release: R1.
- As a household member, I want to see inflows and outflows by purpose for a period, so
  that I know where money comes from and where it goes.

Acceptance criteria:

- Given flows in October 2026, when I view in and out for October 2026, then in and out
  are netted per purpose, purposes with net in are listed as income and purposes with
  net out as spending, and flows without a purpose are shown under "Unclassified" as
  in and out separately (ADR-0022).
- Given spending of EUR 54.20 at Corner Grocer and a refund of EUR 20.00 from Corner
  Grocer in the same period, both with the purpose "Groceries", then Groceries shows
  spending of EUR 34.20 (ADR-0022).
- Given a purpose whose in and out net to exactly zero in the period, then it is
  listed in a group of its own, "Netted to zero", below income and spending, with its
  gross in and out visible on expansion. It never disappears.
- Given a transaction whose flows are all on household accounts (a transfer), then it
  adds nothing to the household view.
- Given a transaction with no flow on a household account, then it appears in no in
  and out view (ADR-0013).
- Given flows on a system party's account (e.g. the opening party's, for opening
  balances), then they are neither in nor out. They appear on that party's line of
  the reconciliation, "Opening balances" (ADR-0020).
- Given spending of EUR 54.20 from the joint account with shares 50/50, when I view per
  member, then each member shows out EUR 27.10.
- Given spending of EUR 54.21 from that account, then one member shows out EUR 27.11
  and the other EUR 27.10, and between members is zero for both (ADR-0019).
- Given income of EUR 3,800.00 from ACME GmbH, of which EUR 3,000.00 arrive on Alex's
  current account and EUR 800.00 on the joint account, then in is EUR 3,400.00 for
  Alex and EUR 400.00 for Sam, and between members is zero (ADR-0018).
- Given a cash withdrawal of EUR 200.00 from the joint account into Alex's cash wallet
  with a EUR 2.00 fee, then out is EUR 1.00 for each member, and between members is
  EUR +100.00 for Alex and EUR -100.00 for Sam (ADR-0018).
- Given income of EUR 3,800.00 from ACME GmbH from which Example Bank withholds a fee
  of EUR 5.00, so that EUR 3,795.00 arrive on Alex's current account, then Alex shows
  in EUR 3,800.00 and out EUR 5.00 (ADR-0018, no household flow of the opposite
  sign).
- Given a period that starts on or before the date of an account's opening balance,
  then the view is flagged partial once, with a note naming the account as not
  tracked before that date. Purpose lines are not flagged one by one (ADR-0021).
- Given an aggregate flow for 2026-10-01 to 2026-10-31, when I view October 2026 or any
  period containing it, then it counts in full.
- Given the same aggregate flow, when I view 2026-10-15 to 2026-11-15, then it is
  excluded, and the line of its purpose ("Unclassified" if it has none) and the
  totals are flagged partial, naming the aggregate (ADR-0021).
- Given the October example in ADR-0012 with opening balances at 2026-09-30, when I view
  per member, then Alex shows opening 2,000, in 3,000, out 300, between members -500,
  closing 4,200, and Sam shows opening 2,000, in 2,500, out 300, between members
  +500, closing 4,700.
- Given Sam's cash wallet with an opening balance of EUR 80.00 at 2026-10-10 and all
  other accounts with opening balances at 2026-09-30, when I view October 2026, then
  the opening figure leaves the wallet out and names it as not tracked, the line
  "Opening balances" shows EUR 80.00 for Sam and for the household, and the view
  reconciles (ADR-0020).
- Given any period, then each member reconciles (opening + opening balances + in - out
  + between members = closing), between-members lines sum to zero across the
  household, and members' in and out sum to the household's in and out, all in whole
  cents (ADR-0019).

### S-014: Sign in

- Activity: -. Release: R1.
- As a household member, I want to reach Iasion with my existing Cloudflare Access login,
  so that the household's data is not exposed and I do not manage another password.

Acceptance criteria:

- Given a request with a valid Cloudflare Access JWT for an email that is a login
  identity of a member, then the request is served in that member's context.
- Given a request without a valid Access JWT, then it is rejected, even if an email
  header is present.
- Given a valid Access JWT for an email not mapped to any member, then access is denied.
- Given member Sam without a login identity, when I map an email to Sam as a login
  identity, then requests with a valid Access JWT for that email are served in Sam's
  context (ADR-0024).
- Given a login identity that is mapped to a member, when I map it to another member,
  then it is rejected: a login identity belongs to one member. A member may have
  several login identities, or none.
- Given a household with two login identities, when I remove one, then requests with a
  valid Access JWT for that email are denied afterwards.
- Given a household with exactly one login identity, when I remove it, then it is
  rejected with a typed error: a household always has at least one login identity
  (ADR-0026).
- Given a household exists, then the email named for first setup has no role: a
  request for it is served only if that email is a login identity of a member
  (ADR-0026).

### S-015: Correct household data

- Activity: A1. Task: A1.8 Correct the past. Release: R1.
- As a household member, I want to fix a party, an account, ownership shares or a
  purpose I entered wrongly, so that mistakes there do not stay in the figures either.

Acceptance criteria:

- Given party "Corner Grocer", when I correct its name, then views use the new name and
  the earlier version is still stored with its recorded time.
- Given the joint current account with shares 1/2 and 1/2, when I correct them to 3/5
  and 2/5, then every member view uses 3/5 and 2/5 for all dates, including dates
  before the correction was recorded.
- Given that correction, when I view figures as known on a time before it was recorded,
  then they use 1/2 and 1/2.
- Given corrected shares that do not sum to 1, then the correction is rejected with a
  typed error naming the actual sum.
- Given a party of kind legal entity, when I correct its kind to system, then it is
  rejected. The kind of a system party cannot be corrected either.
- Given member Sam, when I try to make Sam a party outside the boundary, or party
  "ACME GmbH" a member, then it is rejected.
- Given a purpose with a current assignment, when I void it, then it is rejected with a
  typed error naming the facts that refer to it.
- Given a purpose that no current fact refers to, when I void it, then it can no longer
  be assigned, and its versions are still stored.
- Given an account with an opening balance, when I void the account, then it is
  rejected. Given an account that no current fact refers to, then it is voided.
- Given a party with a flow on its implicit account, when I void the party, then it is
  rejected. Given a party that no current fact refers to, then it is voided together
  with its implicit account.
