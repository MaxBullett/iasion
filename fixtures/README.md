# Fixtures

Test fixtures for golden and acceptance tests.

- `fixtures/private/` holds real personal documents (bank statements, amortization
  schedules, payslips). It is gitignored and must never be committed.
- Committed fixtures must be synthetic or fully anonymized.
- Tests that depend on private fixtures must skip cleanly when the files are absent.
