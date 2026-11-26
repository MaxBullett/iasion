#!/bin/sh
# Creates the owner and the application role and the database (ADR-0033, decision 7).
# Runs once, as the PostgreSQL superuser: in development as an init script of the `db`
# container (ADR-0038), in CI before the migrations, and on the host at setup
# (ADR-0040). The passwords come from the environment, never from this file.
set -eu

: "${IASION_DB_OWNER_PASSWORD:?is not set (see .env.example)}"
: "${IASION_DB_APP_PASSWORD:?is not set (see .env.example)}"

psql -v ON_ERROR_STOP=1 --username "${POSTGRES_USER:-postgres}" --dbname postgres \
    -v owner_password="$IASION_DB_OWNER_PASSWORD" \
    -v app_password="$IASION_DB_APP_PASSWORD" <<'SQL'
-- The passwords are psql variables set above, not values. betterleaks:allow below.
CREATE ROLE iasion_owner LOGIN PASSWORD :'owner_password'; -- betterleaks:allow
CREATE ROLE iasion_app LOGIN PASSWORD :'app_password'; -- betterleaks:allow
CREATE DATABASE iasion OWNER iasion_owner;
REVOKE ALL ON DATABASE iasion FROM PUBLIC;
GRANT CONNECT ON DATABASE iasion TO iasion_app;
SQL
