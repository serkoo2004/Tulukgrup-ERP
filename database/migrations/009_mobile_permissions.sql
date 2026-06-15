CREATE TABLE IF NOT EXISTS mobile_user_permissions (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  company_id BIGINT REFERENCES companies(id),
  department_id BIGINT REFERENCES departments(id),
  permission_key VARCHAR(120) NOT NULL,
  can_view BOOLEAN NOT NULL DEFAULT true,
  can_create BOOLEAN NOT NULL DEFAULT false,
  can_update BOOLEAN NOT NULL DEFAULT false,
  can_approve BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_mobile_user_permissions_scope
  ON mobile_user_permissions(user_id, COALESCE(company_id, 0), COALESCE(department_id, 0), permission_key);

CREATE INDEX IF NOT EXISTS idx_mobile_user_permissions_user
  ON mobile_user_permissions(user_id, company_id, department_id)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_mobile_user_permissions_key
  ON mobile_user_permissions(permission_key)
  WHERE deleted_at IS NULL;
