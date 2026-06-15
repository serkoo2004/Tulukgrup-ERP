ALTER TABLE notifications
  ADD COLUMN IF NOT EXISTS recipient_phone TEXT,
  ADD COLUMN IF NOT EXISTS external_message_id TEXT,
  ADD COLUMN IF NOT EXISTS provider_payload JSONB,
  ADD COLUMN IF NOT EXISTS delivery_error TEXT;

CREATE INDEX IF NOT EXISTS idx_notifications_channel_status
  ON notifications(sent_via, delivery_status)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_notifications_external_message
  ON notifications(external_message_id)
  WHERE external_message_id IS NOT NULL;
