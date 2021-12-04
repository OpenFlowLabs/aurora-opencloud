-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tenants (
    id uuid PRIMARY KEY default uuid_generate_v4(),
    name varchar NOT NULL
);

CREATE UNIQUE INDEX idx_tenant_name ON tenants(name);