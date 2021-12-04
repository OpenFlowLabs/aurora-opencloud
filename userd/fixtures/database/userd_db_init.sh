#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER userd;
    CREATE DATABASE userd_develop;
    GRANT ALL PRIVILEGES ON DATABASE userd_develop TO userd;
EOSQL

