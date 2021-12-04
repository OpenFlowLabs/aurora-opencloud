-- Create User table and mail verification table

CREATE TABLE users(
    id uuid PRIMARY KEY default uuid_generate_v4(),
    tenant_id uuid NOT NULL,
    username varchar NOT NULL,
    pwhash varchar NOT NULL,
    email varchar NOT NULL,
    email_confirmed BOOLEAN NOT NULL default false,
    UNIQUE (username, tenant_id),
    UNIQUE (email, tenant_id),
    CONSTRAINT fk_user_tenant
        FOREIGN KEY(tenant_id)
        REFERENCES tenants(id)
        ON DELETE NO ACTION
);

CREATE TABLE user_confirmations (
  user_id uuid PRIMARY KEY NOT NULL,
  tenant_id uuid NOT NULL,
  token varchar NOT NULL,
  email varchar NOT NULL,
  UNIQUE (token),
  UNIQUE (email, tenant_id),
  CONSTRAINT fk_confirmation_tenant
      FOREIGN KEY(tenant_id)
          REFERENCES tenants(id)
          ON DELETE NO ACTION,
  CONSTRAINT fk_confirmation_user
      FOREIGN KEY(user_id)
          REFERENCES users(id)
          ON DELETE NO ACTION
);

CREATE INDEX idx_users_tenant ON users(username, tenant_id);
CREATE INDEX idx_users_confirmation_email ON user_confirmations(email, tenant_id);
CREATE INDEX idx_users_confirmation_token ON user_confirmations(token);