-- Schema of the Ledger context. Only its crate reads and writes it (ADR-0033,
-- decision 3). The application role may read and insert in every table created here,
-- and nothing more unless a migration grants it (ADR-0033, decision 7).
CREATE SCHEMA ledger;

GRANT USAGE ON SCHEMA ledger TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA ledger GRANT SELECT, INSERT ON TABLES TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA ledger GRANT USAGE ON SEQUENCES TO iasion_app;
