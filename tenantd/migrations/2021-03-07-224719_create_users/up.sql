-- Create User table and mail verification table

CREATE TABLE principals(
    id uuid PRIMARY KEY default uuid_generate_v4(),
    tenant_id uuid NOT NULL,
    p_name varchar NOT NULL,
    email varchar NULL,
    email_confirmed BOOLEAN NULL,
    UNIQUE (p_name, tenant_id),
    UNIQUE (email, tenant_id),
    CONSTRAINT fk_principal_tenant
        FOREIGN KEY(tenant_id)
        REFERENCES tenants(id)
        ON DELETE NO ACTION
);


CREATE TABLE public_keys(
    fingerprint varchar PRIMARY KEY NOT NULL,
    public_key varchar NOT NULL,
    public_key_pem varchar NOT NULL,
    public_key_paserk VARCHAR NOT NULL
);

CREATE TABLE principals_pks(
    p_id uuid NOT NULL REFERENCES principals(id)
        ON DELETE CASCADE,
    fingerprint varchar NOT NULL REFERENCES public_keys(fingerprint)
        ON DELETE CASCADE,
    PRIMARY KEY (p_id,fingerprint)
);

CREATE TABLE user_confirmations (
  p_id uuid PRIMARY KEY NOT NULL,
  tenant_id uuid NOT NULL,
  token varchar NOT NULL,
  email varchar NOT NULL,
  UNIQUE (token),
  UNIQUE (email, tenant_id),
  CONSTRAINT fk_confirmation_tenant
      FOREIGN KEY(tenant_id)
          REFERENCES tenants(id)
          ON DELETE NO ACTION,
  CONSTRAINT fk_confirmation_principal
      FOREIGN KEY(p_id)
          REFERENCES principals(id)
          ON DELETE NO ACTION
);

CREATE INDEX idx_principals_tenant_name ON principals(p_name, tenant_id);
CREATE INDEX idx_principals_tenant_email ON principals(email, tenant_id);
CREATE INDEX idx_principals_confirmation_email ON user_confirmations(email, tenant_id);
CREATE INDEX idx_principals_confirmation_token ON user_confirmations(token);