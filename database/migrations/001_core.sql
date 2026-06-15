CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE user_role AS ENUM ('admin', 'manager', 'operation', 'accounting', 'user');
CREATE TYPE vehicle_status AS ENUM ('active', 'passive', 'sold');
CREATE TYPE km_entry_type AS ENUM ('manual', 'ocr', 'mobiliz_api', 'kopilot_api');
CREATE TYPE verification_status AS ENUM ('pending', 'verified', 'suspicious', 'rejected');
CREATE TYPE maintenance_status AS ENUM ('planned', 'scheduled', 'completed', 'cancelled');
CREATE TYPE policy_type AS ENUM ('trafik', 'kasko', 'imm', 'ferdi_kaza');
CREATE TYPE renewal_status AS ENUM ('active', 'approaching', 'renewing', 'ended');
CREATE TYPE expense_type AS ENUM ('fuel', 'insurance', 'maintenance', 'tax', 'fine', 'tire', 'service');
CREATE TYPE damage_status AS ENUM ('open', 'expertise', 'insurance', 'repaired', 'closed', 'cancelled');
CREATE TYPE priority_level AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE notification_channel AS ENUM ('whatsapp', 'system', 'email');
CREATE TYPE audit_action AS ENUM ('create', 'update', 'delete', 'restore', 'sell', 'login', 'logout');

CREATE TABLE companies (
  id BIGSERIAL PRIMARY KEY,
  name VARCHAR(160) NOT NULL UNIQUE,
  tax_number VARCHAR(64),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT,
  updated_by BIGINT,
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE departments (
  id BIGSERIAL PRIMARY KEY,
  company_id BIGINT REFERENCES companies(id),
  name VARCHAR(160) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT,
  updated_by BIGINT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  CONSTRAINT uq_department_company_name UNIQUE (company_id, name)
);

CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  email VARCHAR(255) NOT NULL UNIQUE,
  full_name VARCHAR(160) NOT NULL,
  phone VARCHAR(32),
  extension VARCHAR(16),
  password_hash VARCHAR(255) NOT NULL,
  role user_role NOT NULL DEFAULT 'user',
  company_id BIGINT REFERENCES companies(id),
  department_id BIGINT REFERENCES departments(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

ALTER TABLE companies
  ADD CONSTRAINT fk_companies_created_by FOREIGN KEY (created_by) REFERENCES users(id),
  ADD CONSTRAINT fk_companies_updated_by FOREIGN KEY (updated_by) REFERENCES users(id);

ALTER TABLE departments
  ADD CONSTRAINT fk_departments_created_by FOREIGN KEY (created_by) REFERENCES users(id),
  ADD CONSTRAINT fk_departments_updated_by FOREIGN KEY (updated_by) REFERENCES users(id);

CREATE TABLE refresh_tokens (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash VARCHAR(255) NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  revoked_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  user_agent TEXT,
  ip_address INET
);

CREATE TABLE vehicles (
  id BIGSERIAL PRIMARY KEY,
  plate VARCHAR(32) NOT NULL UNIQUE,
  brand VARCHAR(80) NOT NULL,
  model VARCHAR(100) NOT NULL,
  model_year INTEGER,
  vehicle_type VARCHAR(80),
  fuel_type VARCHAR(50),
  transmission VARCHAR(50),
  chassis_no VARCHAR(80),
  engine_no VARCHAR(80),
  warranty_status VARCHAR(80),
  warranty_end DATE,
  has_hgs BOOLEAN NOT NULL DEFAULT false,
  has_mobiliz BOOLEAN NOT NULL DEFAULT false,
  has_kopilot BOOLEAN NOT NULL DEFAULT false,
  has_k2 BOOLEAN NOT NULL DEFAULT false,
  tasitmatik_company VARCHAR(120),
  spare_key_location VARCHAR(160),
  status vehicle_status NOT NULL DEFAULT 'active',
  company_id BIGINT REFERENCES companies(id),
  department_id BIGINT REFERENCES departments(id),
  user_id BIGINT REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE sold_vehicles (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL UNIQUE REFERENCES vehicles(id),
  sold_date DATE NOT NULL,
  sold_reason VARCHAR(160) NOT NULL,
  sold_price NUMERIC(14, 2),
  buyer_info TEXT,
  company_exit_reason VARCHAR(80) NOT NULL,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE vehicle_assignments (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  user_id BIGINT NOT NULL REFERENCES users(id),
  assigned_at TIMESTAMPTZ NOT NULL,
  released_at TIMESTAMPTZ,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE km_logs (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  km INTEGER NOT NULL CHECK (km >= 0),
  entry_type km_entry_type NOT NULL DEFAULT 'manual',
  image_path TEXT,
  ocr_result JSONB,
  verification_status verification_status NOT NULL DEFAULT 'pending',
  ip_address INET,
  device_info TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE maintenances (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  last_maintenance_km INTEGER,
  next_maintenance_km INTEGER,
  maintenance_date DATE,
  service_company VARCHAR(160),
  maintenance_type VARCHAR(120),
  description TEXT,
  total_cost NUMERIC(14, 2),
  invoice_file TEXT,
  maintenance_status maintenance_status NOT NULL DEFAULT 'planned',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE insurance_policies (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  policy_type policy_type NOT NULL,
  policy_number VARCHAR(120) NOT NULL,
  insurance_company VARCHAR(160),
  agency_name VARCHAR(160),
  start_date DATE,
  end_date DATE,
  amount NUMERIC(14, 2),
  previous_amount NUMERIC(14, 2),
  currency VARCHAR(8) NOT NULL DEFAULT 'TRY',
  pdf_file TEXT,
  renewal_status renewal_status NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE expenses (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  expense_type expense_type NOT NULL,
  amount NUMERIC(14, 2) NOT NULL,
  invoice_number VARCHAR(120),
  invoice_file TEXT,
  payment_status VARCHAR(80),
  expense_date DATE NOT NULL,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE damages (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  damage_date DATE NOT NULL,
  damage_type VARCHAR(120),
  description TEXT,
  estimated_cost NUMERIC(14, 2),
  actual_cost NUMERIC(14, 2),
  insurance_claim_no VARCHAR(120),
  damage_status damage_status NOT NULL DEFAULT 'open',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE tasks (
  id BIGSERIAL PRIMARY KEY,
  task_type VARCHAR(120) NOT NULL,
  related_vehicle_id BIGINT REFERENCES vehicles(id),
  assigned_user_id BIGINT REFERENCES users(id),
  assigned_department_id BIGINT REFERENCES departments(id),
  priority priority_level NOT NULL DEFAULT 'medium',
  due_date DATE,
  task_status VARCHAR(60) NOT NULL DEFAULT 'open',
  description TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE notifications (
  id BIGSERIAL PRIMARY KEY,
  notification_type VARCHAR(120) NOT NULL,
  receiver_user_id BIGINT REFERENCES users(id),
  related_vehicle_id BIGINT REFERENCES vehicles(id),
  message TEXT NOT NULL,
  sent_via notification_channel NOT NULL,
  delivery_status VARCHAR(80) NOT NULL DEFAULT 'pending',
  sent_at TIMESTAMPTZ,
  read_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE file_documents (
  id BIGSERIAL PRIMARY KEY,
  module_name VARCHAR(80) NOT NULL,
  entity_id BIGINT NOT NULL,
  file_type VARCHAR(80) NOT NULL,
  original_name TEXT NOT NULL,
  storage_path TEXT NOT NULL,
  mime_type VARCHAR(160),
  file_size BIGINT NOT NULL,
  checksum_sha256 VARCHAR(64),
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE import_jobs (
  id BIGSERIAL PRIMARY KEY,
  target_module VARCHAR(80) NOT NULL,
  source_file_id BIGINT REFERENCES file_documents(id),
  column_mapping JSONB,
  import_status VARCHAR(60) NOT NULL DEFAULT 'pending',
  total_rows INTEGER NOT NULL DEFAULT 0,
  processed_rows INTEGER NOT NULL DEFAULT 0,
  error_count INTEGER NOT NULL DEFAULT 0,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE import_errors (
  id BIGSERIAL PRIMARY KEY,
  import_job_id BIGINT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE,
  row_number INTEGER,
  field_name VARCHAR(120),
  error_message TEXT NOT NULL,
  raw_data JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE import_rows (
  id BIGSERIAL PRIMARY KEY,
  import_job_id BIGINT NOT NULL REFERENCES import_jobs(id) ON DELETE CASCADE,
  row_number INTEGER NOT NULL,
  raw_data JSONB NOT NULL,
  normalized_data JSONB,
  row_status VARCHAR(60) NOT NULL DEFAULT 'pending',
  error_count INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (import_job_id, row_number)
);

CREATE TABLE ai_analysis_jobs (
  id BIGSERIAL PRIMARY KEY,
  analysis_type VARCHAR(120) NOT NULL,
  related_vehicle_id BIGINT REFERENCES vehicles(id),
  source_table VARCHAR(120),
  source_record_id BIGINT,
  input_data JSONB,
  result_data JSONB,
  confidence_score NUMERIC(5, 4),
  job_status VARCHAR(60) NOT NULL DEFAULT 'pending',
  requires_human_approval BOOLEAN NOT NULL DEFAULT true,
  approved_by BIGINT REFERENCES users(id),
  approved_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE audit_logs (
  id BIGSERIAL PRIMARY KEY,
  table_name VARCHAR(120) NOT NULL,
  record_id BIGINT,
  action_type audit_action NOT NULL,
  old_data JSONB,
  new_data JSONB,
  ip_address INET,
  user_agent TEXT,
  created_by BIGINT REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE system_error_logs (
  id BIGSERIAL PRIMARY KEY,
  service_name VARCHAR(120) NOT NULL,
  severity VARCHAR(40) NOT NULL,
  message TEXT NOT NULL,
  context JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE system_settings (
  id BIGSERIAL PRIMARY KEY,
  setting_key VARCHAR(160) NOT NULL UNIQUE,
  setting_value JSONB NOT NULL,
  description TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id)
);

CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_vehicles_status ON vehicles(status);
CREATE INDEX idx_vehicles_company_department ON vehicles(company_id, department_id);
CREATE INDEX idx_km_logs_vehicle_created ON km_logs(vehicle_id, created_at DESC);
CREATE INDEX idx_maintenances_vehicle_status ON maintenances(vehicle_id, maintenance_status);
CREATE INDEX idx_policies_vehicle_end ON insurance_policies(vehicle_id, end_date);
CREATE INDEX idx_expenses_vehicle_date ON expenses(vehicle_id, expense_date DESC);
CREATE INDEX idx_damages_vehicle_date ON damages(vehicle_id, damage_date DESC);
CREATE INDEX idx_damages_status ON damages(damage_status);
CREATE INDEX idx_tasks_vehicle_status ON tasks(related_vehicle_id, task_status);
CREATE INDEX idx_notifications_receiver_read ON notifications(receiver_user_id, read_at);
CREATE INDEX idx_file_documents_module_entity ON file_documents(module_name, entity_id);
CREATE INDEX idx_file_documents_type ON file_documents(file_type);
CREATE INDEX idx_import_jobs_target_status ON import_jobs(target_module, import_status);
CREATE INDEX idx_import_errors_job ON import_errors(import_job_id);
CREATE INDEX idx_import_rows_job_status ON import_rows(import_job_id, row_status);
CREATE INDEX idx_ai_analysis_jobs_type_status ON ai_analysis_jobs(analysis_type, job_status);
CREATE INDEX idx_ai_analysis_jobs_vehicle ON ai_analysis_jobs(related_vehicle_id);
CREATE INDEX idx_audit_logs_table_record ON audit_logs(table_name, record_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);
CREATE INDEX idx_system_settings_active ON system_settings(is_active);
