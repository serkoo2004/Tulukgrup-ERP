CREATE TABLE inventory_products (
    id BIGSERIAL PRIMARY KEY,
    product_code TEXT NOT NULL UNIQUE,
    barcode TEXT UNIQUE,
    qr_code TEXT UNIQUE,
    product_name TEXT NOT NULL,
    category TEXT,
    sub_category TEXT,
    brand TEXT,
    description TEXT,
    main_unit TEXT NOT NULL DEFAULT 'adet',
    package_unit TEXT,
    package_multiplier NUMERIC(18, 4),
    current_stock NUMERIC(18, 4) NOT NULL DEFAULT 0,
    available_stock NUMERIC(18, 4) NOT NULL DEFAULT 0,
    reserved_stock NUMERIC(18, 4) NOT NULL DEFAULT 0,
    minimum_stock NUMERIC(18, 4) NOT NULL DEFAULT 0,
    maximum_stock NUMERIC(18, 4),
    critical_stock NUMERIC(18, 4),
    safety_stock NUMERIC(18, 4),
    unit_cost NUMERIC(18, 4),
    is_lot_tracked BOOLEAN NOT NULL DEFAULT false,
    expiry_tracking BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    created_by BIGINT REFERENCES users(id),
    updated_by BIGINT REFERENCES users(id)
);

CREATE INDEX idx_inventory_products_name ON inventory_products(product_name);
CREATE INDEX idx_inventory_products_category ON inventory_products(category, sub_category);
CREATE INDEX idx_inventory_products_low_stock ON inventory_products(is_active, deleted_at, current_stock, minimum_stock);

CREATE TABLE inventory_lots (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES inventory_products(id),
    lot_number TEXT NOT NULL,
    production_date DATE,
    expiry_date DATE,
    supplier TEXT,
    quantity NUMERIC(18, 4) NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    created_by BIGINT REFERENCES users(id),
    updated_by BIGINT REFERENCES users(id),
    UNIQUE(product_id, lot_number)
);

CREATE INDEX idx_inventory_lots_product ON inventory_lots(product_id);
CREATE INDEX idx_inventory_lots_expiry ON inventory_lots(expiry_date) WHERE deleted_at IS NULL;

CREATE TABLE inventory_movements (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES inventory_products(id),
    lot_id BIGINT REFERENCES inventory_lots(id),
    movement_type TEXT NOT NULL,
    quantity NUMERIC(18, 4) NOT NULL,
    unit TEXT NOT NULL DEFAULT 'adet',
    unit_multiplier NUMERIC(18, 4) NOT NULL DEFAULT 1,
    base_quantity NUMERIC(18, 4) NOT NULL,
    previous_stock NUMERIC(18, 4) NOT NULL,
    next_stock NUMERIC(18, 4) NOT NULL,
    branch_id BIGINT REFERENCES departments(id),
    vehicle_id BIGINT REFERENCES vehicles(id),
    reference_table TEXT,
    reference_id BIGINT,
    description TEXT,
    movement_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    cancelled_at TIMESTAMPTZ,
    cancel_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by BIGINT REFERENCES users(id)
);

CREATE INDEX idx_inventory_movements_product ON inventory_movements(product_id, movement_at DESC);
CREATE INDEX idx_inventory_movements_type ON inventory_movements(movement_type, movement_at DESC);
CREATE INDEX idx_inventory_movements_vehicle ON inventory_movements(vehicle_id) WHERE vehicle_id IS NOT NULL;
CREATE INDEX idx_inventory_movements_branch ON inventory_movements(branch_id) WHERE branch_id IS NOT NULL;

CREATE TABLE inventory_shipments (
    id BIGSERIAL PRIMARY KEY,
    branch_id BIGINT REFERENCES departments(id),
    shipment_status TEXT NOT NULL DEFAULT 'draft',
    sender_user_id BIGINT REFERENCES users(id),
    approver_user_id BIGINT REFERENCES users(id),
    shipped_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    created_by BIGINT REFERENCES users(id),
    updated_by BIGINT REFERENCES users(id)
);

CREATE TABLE inventory_shipment_items (
    id BIGSERIAL PRIMARY KEY,
    shipment_id BIGINT NOT NULL REFERENCES inventory_shipments(id) ON DELETE CASCADE,
    product_id BIGINT NOT NULL REFERENCES inventory_products(id),
    quantity NUMERIC(18, 4) NOT NULL,
    unit TEXT NOT NULL DEFAULT 'adet',
    unit_multiplier NUMERIC(18, 4) NOT NULL DEFAULT 1,
    base_quantity NUMERIC(18, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_inventory_shipments_status ON inventory_shipments(shipment_status, created_at DESC);
CREATE INDEX idx_inventory_shipment_items_shipment ON inventory_shipment_items(shipment_id);

CREATE TABLE inventory_counts (
    id BIGSERIAL PRIMARY KEY,
    count_type TEXT NOT NULL,
    count_method TEXT NOT NULL DEFAULT 'manual',
    category TEXT,
    count_status TEXT NOT NULL DEFAULT 'draft',
    note TEXT,
    counted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    created_by BIGINT REFERENCES users(id),
    updated_by BIGINT REFERENCES users(id)
);

CREATE TABLE inventory_count_items (
    id BIGSERIAL PRIMARY KEY,
    count_id BIGINT NOT NULL REFERENCES inventory_counts(id) ON DELETE CASCADE,
    product_id BIGINT NOT NULL REFERENCES inventory_products(id),
    system_stock NUMERIC(18, 4) NOT NULL,
    counted_stock NUMERIC(18, 4) NOT NULL,
    difference NUMERIC(18, 4) NOT NULL,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_inventory_counts_status ON inventory_counts(count_status, created_at DESC);
CREATE INDEX idx_inventory_count_items_count ON inventory_count_items(count_id);

CREATE TABLE inventory_purchase_requests (
    id BIGSERIAL PRIMARY KEY,
    product_id BIGINT NOT NULL REFERENCES inventory_products(id),
    request_status TEXT NOT NULL DEFAULT 'open',
    recommended_quantity NUMERIC(18, 4) NOT NULL,
    reason TEXT,
    monthly_consumption_avg NUMERIC(18, 4),
    estimated_runout_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    created_by BIGINT REFERENCES users(id),
    updated_by BIGINT REFERENCES users(id)
);

CREATE INDEX idx_inventory_purchase_requests_status ON inventory_purchase_requests(request_status, created_at DESC);
