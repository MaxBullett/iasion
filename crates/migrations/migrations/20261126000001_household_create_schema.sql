-- Schema of the Household context. Only its crate reads and writes it (ADR-0033,
-- decision 3). The application role may read and insert in every table created here,
-- and nothing more unless a migration grants it (ADR-0033, decision 7).
CREATE SCHEMA household;

GRANT USAGE ON SCHEMA household TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA household GRANT SELECT, INSERT ON TABLES TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA household GRANT USAGE ON SEQUENCES TO iasion_app;
