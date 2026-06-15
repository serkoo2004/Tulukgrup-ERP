CREATE TABLE vehicle_inspections (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  branch VARCHAR(120),
  inspection_date DATE NOT NULL,
  inspector_user_id BIGINT REFERENCES users(id),
  exterior_ok BOOLEAN,
  interior_ok BOOLEAN,
  equipment_ok BOOLEAN,
  documents_ok BOOLEAN,
  damage_note TEXT,
  action_note TEXT,
  expert_report TEXT,
  fee NUMERIC(14, 2),
  inspection_status VARCHAR(60) NOT NULL DEFAULT 'open',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE value_loss_claims (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  accident_date DATE NOT NULL,
  vehicle_purchase_date DATE,
  tramer_amount NUMERIC(14, 2),
  deprivation_days INTEGER,
  requested_amount NUMERIC(14, 2),
  received_amount NUMERIC(14, 2),
  claim_status VARCHAR(80) NOT NULL DEFAULT 'open',
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE fuel_entries (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  period_year INTEGER NOT NULL,
  period_month INTEGER NOT NULL CHECK (period_month BETWEEN 1 AND 12),
  fuel_limit NUMERIC(14, 2),
  paid_amount NUMERIC(14, 2),
  remaining_limit NUMERIC(14, 2),
  distance_km INTEGER,
  current_km INTEGER,
  liter_amount NUMERIC(14, 2),
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true,
  CONSTRAINT uq_fuel_entry_vehicle_period UNIQUE (vehicle_id, period_year, period_month)
);

CREATE TABLE vehicle_washes (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  branch VARCHAR(120),
  wash_company VARCHAR(160),
  wash_date DATE NOT NULL,
  amount NUMERIC(14, 2),
  user_id BIGINT REFERENCES users(id),
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE insurance_quotes (
  id BIGSERIAL PRIMARY KEY,
  vehicle_id BIGINT NOT NULL REFERENCES vehicles(id),
  quote_type VARCHAR(80) NOT NULL DEFAULT 'kasko',
  insurance_company VARCHAR(160),
  agency_name VARCHAR(160),
  gross_premium NUMERIC(14, 2),
  installment_count INTEGER,
  quote_status VARCHAR(80) NOT NULL DEFAULT 'pending',
  valid_until DATE,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_vehicle_inspections_vehicle_date ON vehicle_inspections(vehicle_id, inspection_date DESC);
CREATE INDEX idx_value_loss_claims_vehicle_status ON value_loss_claims(vehicle_id, claim_status);
CREATE INDEX idx_fuel_entries_vehicle_period ON fuel_entries(vehicle_id, period_year DESC, period_month DESC);
CREATE INDEX idx_vehicle_washes_vehicle_date ON vehicle_washes(vehicle_id, wash_date DESC);
CREATE INDEX idx_insurance_quotes_vehicle_status ON insurance_quotes(vehicle_id, quote_status);
