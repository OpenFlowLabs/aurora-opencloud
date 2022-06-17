#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 -h db --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER tenantd;
    CREATE DATABASE tenantd_develop;
    GRANT ALL PRIVILEGES ON DATABASE tenantd_develop TO tenantd;
EOSQL
