# Fixtures

Test fixtures for golden and acceptance tests.

- `fixtures/private/` holds real personal documents (bank statements, amortization
  schedules, payslips). It is gitignored and must never be committed.
- Committed fixtures must be synthetic or fully anonymized.
- Tests that depend on private fixtures must skip cleanly when the files are absent.

See "Personal data and secrets" in `AGENTS.md`.

## Reference household

All examples in specs, stories, tests and committed fixtures use this fictional
household. Extend it here when a new example needs more; do not invent people
elsewhere.

| Party | Kind | Notes |
|---|---|---|
| Alex | natural person, member | |
| Sam | natural person, member | |
| ACME GmbH | legal entity | Employer |
| Corner Grocer | legal entity | Merchant |
| Grocery merchants | aggregate | For aggregate flows |
| Example Bank | institution | Holds the bank accounts |

| Account | Institution | Owners |
|---|---|---|
| Joint current account | Example Bank | Alex 50%, Sam 50% |
| Alex's current account | Example Bank | Alex 100% |
| Sam's current account | Example Bank | Sam 100% |
| Alex's cash wallet | none | Alex 100% |
| Sam's cash wallet | none | Sam 100% |

All amounts used with the reference household are synthetic.
