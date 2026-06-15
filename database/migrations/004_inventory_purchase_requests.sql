ALTER TABLE inventory_purchase_requests
    ADD COLUMN IF NOT EXISTS requested_quantity NUMERIC(18, 4);

UPDATE inventory_purchase_requests
SET requested_quantity = recommended_quantity
WHERE requested_quantity IS NULL;

ALTER TABLE inventory_purchase_requests
    ALTER COLUMN requested_quantity SET NOT NULL;

ALTER TABLE inventory_purchase_requests
    ADD COLUMN IF NOT EXISTS approved_quantity NUMERIC(18, 4);

ALTER TABLE inventory_purchase_requests
    ADD COLUMN IF NOT EXISTS note TEXT;

ALTER TABLE inventory_purchase_requests
    ALTER COLUMN recommended_quantity SET DEFAULT 0;
