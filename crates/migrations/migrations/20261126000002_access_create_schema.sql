-- Schema of the Access context. Only its crate reads and writes it (ADR-0033,
-- decision 3). The application role may read and insert in every table created here,
-- and nothing more unless a migration grants it (ADR-0033, decision 7).
CREATE SCHEMA access;

GRANT USAGE ON SCHEMA access TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA access GRANT SELECT, INSERT ON TABLES TO iasion_app;

ALTER DEFAULT PRIVILEGES IN SCHEMA access GRANT USAGE ON SEQUENCES TO iasion_app;
