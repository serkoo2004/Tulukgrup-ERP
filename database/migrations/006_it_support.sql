CREATE TABLE IF NOT EXISTS support_tickets (
  id BIGSERIAL PRIMARY KEY,
  ticket_no VARCHAR(40) NOT NULL UNIQUE,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  category VARCHAR(80) NOT NULL DEFAULT 'it_support',
  priority priority_level NOT NULL DEFAULT 'medium',
  ticket_status VARCHAR(60) NOT NULL DEFAULT 'open',
  source_channel VARCHAR(40) NOT NULL DEFAULT 'web',
  reporter_name TEXT,
  reporter_phone VARCHAR(60),
  reporter_user_id BIGINT REFERENCES users(id),
  assigned_user_id BIGINT REFERENCES users(id),
  assigned_department_id BIGINT REFERENCES departments(id),
  source_notification_id BIGINT REFERENCES notifications(id),
  external_message_id TEXT,
  resolution_note TEXT,
  resolved_at TIMESTAMPTZ,
  closed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE IF NOT EXISTS support_ticket_events (
  id BIGSERIAL PRIMARY KEY,
  ticket_id BIGINT NOT NULL REFERENCES support_tickets(id) ON DELETE CASCADE,
  event_type VARCHAR(80) NOT NULL,
  note TEXT,
  old_status VARCHAR(60),
  new_status VARCHAR(60),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by BIGINT REFERENCES users(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_support_tickets_external_message
  ON support_tickets(external_message_id)
  WHERE external_message_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_support_tickets_status_priority
  ON support_tickets(ticket_status, priority, created_at DESC)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_support_tickets_reporter_phone
  ON support_tickets(reporter_phone)
  WHERE deleted_at IS NULL;

