CREATE TABLE IF NOT EXISTS support_knowledge_base (
  id BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  category VARCHAR(80) NOT NULL DEFAULT 'it_support',
  problem TEXT NOT NULL,
  solution TEXT NOT NULL,
  tags JSONB NOT NULL DEFAULT '[]'::jsonb,
  source_ticket_id BIGINT REFERENCES support_tickets(id),
  is_published BOOLEAN NOT NULL DEFAULT true,
  view_count BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ,
  created_by BIGINT REFERENCES users(id),
  updated_by BIGINT REFERENCES users(id),
  is_active BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_support_knowledge_base_category
  ON support_knowledge_base(category, created_at DESC)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_support_knowledge_base_source_ticket
  ON support_knowledge_base(source_ticket_id)
  WHERE deleted_at IS NULL;
