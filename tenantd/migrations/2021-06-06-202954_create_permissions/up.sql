-- Create Permission tables

CREATE TABLE policies (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    name varchar not null,
    CONSTRAINT fk_user_tenant_policy_permissions
        FOREIGN KEY(tenant_id)
            REFERENCES tenants(id)
            ON DELETE NO ACTION
);

CREATE TABLE policy_permissions (
    policy_id uuid NOT NULL,
    tenant_id uuid NOT NULL,
    permission varchar NOT NULL,
    PRIMARY KEY(policy_id, permission),
    CONSTRAINT fk_permission_policy
        FOREIGN KEY(policy_id)
            REFERENCES policies(id)
            ON DELETE NO ACTION,
    CONSTRAINT fk_user_tenant_policy_permissions
        FOREIGN KEY(tenant_id)
            REFERENCES tenants(id)
            ON DELETE NO ACTION
);

CREATE TABLE user_permissions (
    tenant_id uuid NOT NULL,
    user_id uuid NOT NULL,
    permission varchar NOT NULL,
    PRIMARY KEY (user_id, tenant_id, permission),
    CONSTRAINT fk_tenant_permissions
        FOREIGN KEY(tenant_id)
        REFERENCES tenants(id)
        ON DELETE NO ACTION,
    CONSTRAINT fk_user_permissions
        FOREIGN KEY(user_id)
        REFERENCES users(id)
        ON DELETE NO ACTION
);

CREATE TABLE known_permissions (
  tenant_id uuid NOT NULL,
  permission varchar NOT NULL,
  PRIMARY KEY (tenant_id, permission),
  CONSTRAINT fk_tenant_permissions
      FOREIGN KEY(tenant_id)
          REFERENCES tenants(id)
          ON DELETE NO ACTION
);