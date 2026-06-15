ALTER TABLE support_tickets
  ADD COLUMN IF NOT EXISTS sla_response_due_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS sla_resolution_due_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS first_response_at TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS escalation_level INTEGER NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS satisfaction_score INTEGER,
  ADD COLUMN IF NOT EXISTS satisfaction_note TEXT;

UPDATE support_tickets
SET
  sla_response_due_at = COALESCE(
    sla_response_due_at,
    created_at + CASE priority::text
      WHEN 'critical' THEN INTERVAL '1 hour'
      WHEN 'high' THEN INTERVAL '4 hours'
      WHEN 'medium' THEN INTERVAL '8 hours'
      ELSE INTERVAL '24 hours'
    END
  ),
  sla_resolution_due_at = COALESCE(
    sla_resolution_due_at,
    created_at + CASE priority::text
      WHEN 'critical' THEN INTERVAL '4 hours'
      WHEN 'high' THEN INTERVAL '24 hours'
      WHEN 'medium' THEN INTERVAL '72 hours'
      ELSE INTERVAL '120 hours'
    END
  )
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_support_tickets_sla_resolution
  ON support_tickets(sla_resolution_due_at)
  WHERE deleted_at IS NULL AND ticket_status NOT IN ('resolved', 'closed', 'cancelled');

CREATE INDEX IF NOT EXISTS idx_support_tickets_sla_response
  ON support_tickets(sla_response_due_at)
  WHERE deleted_at IS NULL AND first_response_at IS NULL;

CREATE OR REPLACE FUNCTION support_sla_status(
  ticket_status TEXT,
  first_response_at TIMESTAMPTZ,
  sla_response_due_at TIMESTAMPTZ,
  sla_resolution_due_at TIMESTAMPTZ
) RETURNS TEXT AS $$
BEGIN
  IF ticket_status IN ('resolved', 'closed', 'cancelled') THEN
    RETURN 'closed';
  END IF;

  IF sla_resolution_due_at IS NOT NULL AND sla_resolution_due_at < now() THEN
    RETURN 'resolution_overdue';
  END IF;

  IF first_response_at IS NULL AND sla_response_due_at IS NOT NULL AND sla_response_due_at < now() THEN
    RETURN 'response_overdue';
  END IF;

  IF sla_resolution_due_at IS NOT NULL AND sla_resolution_due_at <= now() + INTERVAL '4 hours' THEN
    RETURN 'due_soon';
  END IF;

  RETURN 'on_track';
END;
$$ LANGUAGE plpgsql STABLE;
