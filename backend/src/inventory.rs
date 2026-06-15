use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(get_inventory_dashboard))
        .route("/alerts", get(list_stock_alerts))
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product)
                .patch(update_product)
                .delete(archive_product),
        )
        .route("/lots", get(list_lots).post(create_lot))
        .route("/movements", get(list_movements).post(create_movement))
        .route("/movements/:id/cancel", post(cancel_movement))
        .route("/shipments", get(list_shipments).post(create_shipment))
        .route("/shipments/:id", get(get_shipment))
        .route("/shipments/:id/approve", post(approve_shipment))
        .route("/counts", get(list_counts).post(create_count))
        .route("/counts/:id", get(get_count))
        .route("/counts/:id/complete", post(complete_count))
        .route(
            "/purchase-requests",
            get(list_purchase_requests).post(create_purchase_request),
        )
        .route(
            "/purchase-requests/:id",
            get(get_purchase_request).patch(update_purchase_request),
        )
        .route("/purchase-suggestions", get(list_purchase_suggestions))
}

#[derive(Debug, Deserialize)]
struct ProductQuery {
    q: Option<String>,
    category: Option<String>,
    low_stock: Option<bool>,
    include_inactive: Option<bool>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ProductCreate {
    product_code: Option<String>,
    barcode: Option<String>,
    qr_code: Option<String>,
    product_name: String,
    category: Option<String>,
    sub_category: Option<String>,
    brand: Option<String>,
    description: Option<String>,
    main_unit: Option<String>,
    package_unit: Option<String>,
    package_multiplier: Option<Decimal>,
    minimum_stock: Option<Decimal>,
    maximum_stock: Option<Decimal>,
    critical_stock: Option<Decimal>,
    safety_stock: Option<Decimal>,
    unit_cost: Option<Decimal>,
    is_lot_tracked: Option<bool>,
    expiry_tracking: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ProductUpdate {
    barcode: Option<String>,
    qr_code: Option<String>,
    product_name: Option<String>,
    category: Option<String>,
    sub_category: Option<String>,
    brand: Option<String>,
    description: Option<String>,
    main_unit: Option<String>,
    package_unit: Option<String>,
    package_multiplier: Option<Decimal>,
    minimum_stock: Option<Decimal>,
    maximum_stock: Option<Decimal>,
    critical_stock: Option<Decimal>,
    safety_stock: Option<Decimal>,
    unit_cost: Option<Decimal>,
    is_lot_tracked: Option<bool>,
    expiry_tracking: Option<bool>,
    is_active: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct ProductRow {
    id: i64,
    product_code: String,
    barcode: Option<String>,
    qr_code: Option<String>,
    product_name: String,
    category: Option<String>,
    sub_category: Option<String>,
    brand: Option<String>,
    description: Option<String>,
    main_unit: String,
    package_unit: Option<String>,
    package_multiplier: Option<Decimal>,
    current_stock: Decimal,
    available_stock: Decimal,
    reserved_stock: Decimal,
    minimum_stock: Decimal,
    maximum_stock: Option<Decimal>,
    critical_stock: Option<Decimal>,
    safety_stock: Option<Decimal>,
    unit_cost: Option<Decimal>,
    is_lot_tracked: bool,
    expiry_tracking: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MovementQuery {
    product_id: Option<i64>,
    movement_type: Option<String>,
    branch_id: Option<i64>,
    vehicle_id: Option<i64>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MovementCreate {
    product_id: i64,
    lot_id: Option<i64>,
    movement_type: String,
    quantity: Decimal,
    unit: Option<String>,
    unit_multiplier: Option<Decimal>,
    branch_id: Option<i64>,
    vehicle_id: Option<i64>,
    reference_table: Option<String>,
    reference_id: Option<i64>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CancelMovementRequest {
    reason: String,
}

#[derive(Debug, FromRow, Serialize)]
struct MovementRow {
    id: i64,
    product_id: i64,
    product_name: String,
    product_code: String,
    lot_id: Option<i64>,
    lot_number: Option<String>,
    movement_type: String,
    quantity: Decimal,
    unit: String,
    unit_multiplier: Decimal,
    base_quantity: Decimal,
    previous_stock: Decimal,
    next_stock: Decimal,
    branch_id: Option<i64>,
    branch_name: Option<String>,
    vehicle_id: Option<i64>,
    plate: Option<String>,
    reference_table: Option<String>,
    reference_id: Option<i64>,
    description: Option<String>,
    movement_at: DateTime<Utc>,
    cancelled_at: Option<DateTime<Utc>>,
    cancel_reason: Option<String>,
    created_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LotQuery {
    product_id: Option<i64>,
    expiring_days: Option<i64>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct LotCreate {
    product_id: i64,
    lot_number: String,
    production_date: Option<NaiveDate>,
    expiry_date: Option<NaiveDate>,
    supplier: Option<String>,
    quantity: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct LotRow {
    id: i64,
    product_id: i64,
    product_name: String,
    lot_number: String,
    production_date: Option<NaiveDate>,
    expiry_date: Option<NaiveDate>,
    supplier: Option<String>,
    quantity: Decimal,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ShipmentCreate {
    branch_id: Option<i64>,
    sender_user_id: Option<i64>,
    note: Option<String>,
    items: Vec<ShipmentItemInput>,
}

#[derive(Debug, Deserialize)]
struct ShipmentItemInput {
    product_id: i64,
    quantity: Decimal,
    unit: Option<String>,
    unit_multiplier: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct ShipmentRow {
    id: i64,
    branch_id: Option<i64>,
    branch_name: Option<String>,
    shipment_status: String,
    sender_user_id: Option<i64>,
    approver_user_id: Option<i64>,
    shipped_at: Option<DateTime<Utc>>,
    approved_at: Option<DateTime<Utc>>,
    note: Option<String>,
    item_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct ShipmentItemRow {
    id: i64,
    shipment_id: i64,
    product_id: i64,
    product_name: String,
    quantity: Decimal,
    unit: String,
    unit_multiplier: Decimal,
    base_quantity: Decimal,
}

#[derive(Debug, Serialize)]
struct ShipmentDetail {
    shipment: ShipmentRow,
    items: Vec<ShipmentItemRow>,
}

#[derive(Debug, Deserialize)]
struct CountCreate {
    count_type: String,
    count_method: Option<String>,
    category: Option<String>,
    note: Option<String>,
    items: Vec<CountItemInput>,
}

#[derive(Debug, Deserialize)]
struct CountItemInput {
    product_id: i64,
    counted_stock: Decimal,
    reason: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct CountRow {
    id: i64,
    count_type: String,
    count_method: String,
    category: Option<String>,
    count_status: String,
    note: Option<String>,
    counted_at: Option<DateTime<Utc>>,
    item_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct CountItemRow {
    id: i64,
    count_id: i64,
    product_id: i64,
    product_name: String,
    system_stock: Decimal,
    counted_stock: Decimal,
    difference: Decimal,
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct CountDetail {
    count: CountRow,
    items: Vec<CountItemRow>,
}

#[derive(Debug, Deserialize)]
struct PurchaseRequestCreate {
    product_id: i64,
    requested_quantity: Decimal,
    reason: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PurchaseRequestUpdate {
    requested_quantity: Option<Decimal>,
    approved_quantity: Option<Decimal>,
    request_status: Option<String>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct PurchaseRequestRow {
    id: i64,
    product_id: i64,
    product_name: String,
    requested_quantity: Decimal,
    approved_quantity: Option<Decimal>,
    request_status: String,
    reason: Option<String>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct StockAlertRow {
    product_id: i64,
    product_code: String,
    product_name: String,
    current_stock: Decimal,
    minimum_stock: Decimal,
    critical_stock: Option<Decimal>,
    alert_level: String,
    suggested_order_quantity: Decimal,
}

#[derive(Debug, FromRow, Serialize)]
struct PurchaseSuggestionRow {
    product_id: i64,
    product_code: String,
    product_name: String,
    current_stock: Decimal,
    monthly_consumption_3m: Decimal,
    monthly_consumption_6m: Decimal,
    monthly_consumption_12m: Decimal,
    average_monthly_consumption: Decimal,
    estimated_runout_date: Option<NaiveDate>,
    suggested_order_quantity: Decimal,
}

#[derive(Debug, FromRow)]
struct InventoryDashboardSummary {
    total_products: i64,
    total_stock_value: Option<Decimal>,
    critical_stock_count: i64,
    out_of_stock_count: i64,
    pending_shipments: i64,
    pending_purchase_requests: i64,
    expiring_lots_30_days: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct InventoryNotificationRow {
    id: i64,
    notification_type: String,
    message: String,
    sent_via: String,
    delivery_status: String,
    created_at: DateTime<Utc>,
    read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
struct InventoryDashboard {
    total_products: i64,
    total_stock_value: Option<Decimal>,
    critical_stock_count: i64,
    out_of_stock_count: i64,
    pending_shipments: i64,
    pending_purchase_requests: i64,
    expiring_lots_30_days: i64,
    recent_movements: Vec<MovementRow>,
    recent_notifications: Vec<InventoryNotificationRow>,
}

async fn list_products(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProductQuery>,
) -> ApiResult<Json<Vec<ProductRow>>> {
    authorize_read(&state, &headers).await?;
    let q = query.q.map(|value| format!("%{}%", value));
    let rows = sqlx::query_as::<_, ProductRow>(
        "SELECT id, product_code, barcode, qr_code, product_name, category, sub_category, brand,
                description, main_unit, package_unit, package_multiplier, current_stock,
                available_stock, reserved_stock, minimum_stock, maximum_stock, critical_stock,
                safety_stock, unit_cost, is_lot_tracked, expiry_tracking, is_active,
                created_at, updated_at, created_by
         FROM inventory_products
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR product_code ILIKE $1 OR barcode ILIKE $1 OR product_name ILIKE $1
                OR category ILIKE $1 OR sub_category ILIKE $1 OR brand ILIKE $1 OR description ILIKE $1)
           AND ($2::text IS NULL OR category = $2)
           AND ($3::boolean = true OR is_active = true)
           AND ($4::boolean = false OR current_stock <= COALESCE(critical_stock, minimum_stock))
         ORDER BY product_name
         LIMIT $5",
    )
    .bind(q)
    .bind(query.category)
    .bind(query.include_inactive.unwrap_or(false))
    .bind(query.low_stock.unwrap_or(false))
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ProductCreate>,
) -> ApiResult<Json<ProductRow>> {
    let user = authorize_manage(&state, &headers).await?;
    validate_unit(payload.main_unit.as_deref().unwrap_or("adet"))?;
    let product_code = payload
        .product_code
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("INV-{}", Utc::now().timestamp_millis()));
    let row = sqlx::query_as::<_, ProductRow>(
        "INSERT INTO inventory_products
         (product_code, barcode, qr_code, product_name, category, sub_category, brand, description,
          main_unit, package_unit, package_multiplier, minimum_stock, maximum_stock, critical_stock,
          safety_stock, unit_cost, is_lot_tracked, expiry_tracking, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 'adet'), $10, $11,
                 COALESCE($12, 0), $13, $14, $15, $16, COALESCE($17, false), COALESCE($18, false), $19)
         RETURNING id, product_code, barcode, qr_code, product_name, category, sub_category, brand,
                   description, main_unit, package_unit, package_multiplier, current_stock,
                   available_stock, reserved_stock, minimum_stock, maximum_stock, critical_stock,
                   safety_stock, unit_cost, is_lot_tracked, expiry_tracking, is_active,
                   created_at, updated_at, created_by",
    )
    .bind(product_code)
    .bind(payload.barcode)
    .bind(payload.qr_code)
    .bind(payload.product_name)
    .bind(payload.category)
    .bind(payload.sub_category)
    .bind(payload.brand)
    .bind(payload.description)
    .bind(payload.main_unit)
    .bind(payload.package_unit)
    .bind(payload.package_multiplier)
    .bind(payload.minimum_stock)
    .bind(payload.maximum_stock)
    .bind(payload.critical_stock)
    .bind(payload.safety_stock)
    .bind(payload.unit_cost)
    .bind(payload.is_lot_tracked)
    .bind(payload.expiry_tracking)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_product_unique_error)?;
    audit_create(
        &state,
        &headers,
        "inventory_products",
        row.id,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn get_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ProductRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_product(&state.pool, id).await?))
}

async fn update_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ProductUpdate>,
) -> ApiResult<Json<ProductRow>> {
    let user = authorize_manage(&state, &headers).await?;
    if let Some(unit) = payload.main_unit.as_deref() {
        validate_unit(unit)?;
    }
    let old = find_product(&state.pool, id).await?;
    let row = sqlx::query_as::<_, ProductRow>(
        "UPDATE inventory_products SET
            barcode = COALESCE($2, barcode),
            qr_code = COALESCE($3, qr_code),
            product_name = COALESCE($4, product_name),
            category = COALESCE($5, category),
            sub_category = COALESCE($6, sub_category),
            brand = COALESCE($7, brand),
            description = COALESCE($8, description),
            main_unit = COALESCE($9, main_unit),
            package_unit = COALESCE($10, package_unit),
            package_multiplier = COALESCE($11, package_multiplier),
            minimum_stock = COALESCE($12, minimum_stock),
            maximum_stock = COALESCE($13, maximum_stock),
            critical_stock = COALESCE($14, critical_stock),
            safety_stock = COALESCE($15, safety_stock),
            unit_cost = COALESCE($16, unit_cost),
            is_lot_tracked = COALESCE($17, is_lot_tracked),
            expiry_tracking = COALESCE($18, expiry_tracking),
            is_active = COALESCE($19, is_active),
            updated_by = $20,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, product_code, barcode, qr_code, product_name, category, sub_category, brand,
                   description, main_unit, package_unit, package_multiplier, current_stock,
                   available_stock, reserved_stock, minimum_stock, maximum_stock, critical_stock,
                   safety_stock, unit_cost, is_lot_tracked, expiry_tracking, is_active,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.barcode)
    .bind(payload.qr_code)
    .bind(payload.product_name)
    .bind(payload.category)
    .bind(payload.sub_category)
    .bind(payload.brand)
    .bind(payload.description)
    .bind(payload.main_unit)
    .bind(payload.package_unit)
    .bind(payload.package_multiplier)
    .bind(payload.minimum_stock)
    .bind(payload.maximum_stock)
    .bind(payload.critical_stock)
    .bind(payload.safety_stock)
    .bind(payload.unit_cost)
    .bind(payload.is_lot_tracked)
    .bind(payload.expiry_tracking)
    .bind(payload.is_active)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(
        &state,
        &headers,
        "inventory_products",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn archive_product(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ProductRow>> {
    let user = authorize_manage(&state, &headers).await?;
    let old = find_product(&state.pool, id).await?;
    let row = sqlx::query_as::<_, ProductRow>(
        "UPDATE inventory_products SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, product_code, barcode, qr_code, product_name, category, sub_category, brand,
                   description, main_unit, package_unit, package_multiplier, current_stock,
                   available_stock, reserved_stock, minimum_stock, maximum_stock, critical_stock,
                   safety_stock, unit_cost, is_lot_tracked, expiry_tracking, is_active,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(
        &state,
        &headers,
        "inventory_products",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_movements(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MovementQuery>,
) -> ApiResult<Json<Vec<MovementRow>>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(query_movements(&state.pool, query).await?))
}

async fn create_movement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MovementCreate>,
) -> ApiResult<Json<MovementRow>> {
    let user = authorize_move(&state, &headers).await?;
    let mut tx = state.pool.begin().await?;
    let movement = apply_movement(&mut tx, &payload, user.id).await?;
    tx.commit().await?;
    audit_create(
        &state,
        &headers,
        "inventory_movements",
        movement.id,
        &movement,
        user.id,
    )
    .await?;
    create_stock_notification_if_needed(&state.pool, &movement, user.id).await?;
    Ok(Json(movement))
}

async fn cancel_movement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<CancelMovementRequest>,
) -> ApiResult<Json<MovementRow>> {
    let user = authorize_manage(&state, &headers).await?;
    let old = find_movement(&state.pool, id).await?;
    if old.cancelled_at.is_some() {
        return Err(ApiError::Conflict("movement already cancelled".to_string()));
    }
    let reverse_type = if stock_delta(&old.movement_type, old.base_quantity) >= Decimal::ZERO {
        "manual_out"
    } else {
        "manual_in"
    };
    let reverse = MovementCreate {
        product_id: old.product_id,
        lot_id: old.lot_id,
        movement_type: reverse_type.to_string(),
        quantity: old.base_quantity,
        unit: Some(old.unit.clone()),
        unit_multiplier: Some(Decimal::ONE),
        branch_id: old.branch_id,
        vehicle_id: old.vehicle_id,
        reference_table: Some("inventory_movements".to_string()),
        reference_id: Some(old.id),
        description: Some(format!("Iptal hareketi: {}", payload.reason)),
    };
    let mut tx = state.pool.begin().await?;
    let _reverse_row = apply_movement(&mut tx, &reverse, user.id).await?;
    sqlx::query(
        "UPDATE inventory_movements SET cancelled_at = now(), cancel_reason = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(payload.reason)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    let row = find_movement(&state.pool, id).await?;
    audit_update(
        &state,
        &headers,
        "inventory_movements",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_lots(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<LotQuery>,
) -> ApiResult<Json<Vec<LotRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, LotRow>(
        "SELECT l.id, l.product_id, p.product_name, l.lot_number, l.production_date, l.expiry_date,
                l.supplier, l.quantity, l.is_active, l.created_at, l.updated_at, l.created_by
         FROM inventory_lots l
         JOIN inventory_products p ON p.id = l.product_id
         WHERE l.deleted_at IS NULL
           AND ($1::bigint IS NULL OR l.product_id = $1)
           AND ($2::bigint IS NULL OR l.expiry_date <= CURRENT_DATE + ($2 || ' days')::interval)
         ORDER BY l.expiry_date ASC NULLS LAST, l.created_at DESC
         LIMIT $3",
    )
    .bind(query.product_id)
    .bind(query.expiring_days)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_lot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LotCreate>,
) -> ApiResult<Json<LotRow>> {
    let user = authorize_manage(&state, &headers).await?;
    let row = sqlx::query_as::<_, LotRow>(
        "INSERT INTO inventory_lots (product_id, lot_number, production_date, expiry_date, supplier, quantity, created_by)
         VALUES ($1, $2, $3, $4, $5, COALESCE($6, 0), $7)
         RETURNING id, product_id, (SELECT product_name FROM inventory_products WHERE id = product_id) AS product_name,
                   lot_number, production_date, expiry_date, supplier, quantity, is_active, created_at, updated_at, created_by",
    )
    .bind(payload.product_id)
    .bind(payload.lot_number)
    .bind(payload.production_date)
    .bind(payload.expiry_date)
    .bind(payload.supplier)
    .bind(payload.quantity)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(&state, &headers, "inventory_lots", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn list_shipments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ShipmentRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, ShipmentRow>(
        "SELECT s.id, s.branch_id, d.name AS branch_name, s.shipment_status, s.sender_user_id,
                s.approver_user_id, s.shipped_at, s.approved_at, s.note,
                COUNT(i.id)::bigint AS item_count, s.created_at, s.updated_at, s.created_by
         FROM inventory_shipments s
         LEFT JOIN departments d ON d.id = s.branch_id
         LEFT JOIN inventory_shipment_items i ON i.shipment_id = s.id
         WHERE s.deleted_at IS NULL
         GROUP BY s.id, d.name
         ORDER BY s.created_at DESC
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_shipment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ShipmentCreate>,
) -> ApiResult<Json<ShipmentRow>> {
    let user = authorize_move(&state, &headers).await?;
    if payload.items.is_empty() {
        return Err(ApiError::BadRequest(
            "shipment items cannot be empty".to_string(),
        ));
    }
    let mut tx = state.pool.begin().await?;
    let shipment_id: i64 = sqlx::query_scalar(
        "INSERT INTO inventory_shipments (branch_id, sender_user_id, note, created_by)
         VALUES ($1, COALESCE($2, $3), $4, $3)
         RETURNING id",
    )
    .bind(payload.branch_id)
    .bind(payload.sender_user_id)
    .bind(user.id)
    .bind(payload.note)
    .fetch_one(&mut *tx)
    .await?;
    for item in payload.items {
        let multiplier = item.unit_multiplier.unwrap_or(Decimal::ONE);
        let base_quantity = item.quantity * multiplier;
        sqlx::query(
            "INSERT INTO inventory_shipment_items (shipment_id, product_id, quantity, unit, unit_multiplier, base_quantity)
             VALUES ($1, $2, $3, COALESCE($4, 'adet'), $5, $6)",
        )
        .bind(shipment_id)
        .bind(item.product_id)
        .bind(item.quantity)
        .bind(item.unit)
        .bind(multiplier)
        .bind(base_quantity)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let row = find_shipment(&state.pool, shipment_id).await?;
    audit_create(
        &state,
        &headers,
        "inventory_shipments",
        row.id,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn get_shipment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ShipmentDetail>> {
    authorize_read(&state, &headers).await?;
    let shipment = find_shipment(&state.pool, id).await?;
    let items = list_shipment_items(&state.pool, id).await?;
    Ok(Json(ShipmentDetail { shipment, items }))
}

async fn approve_shipment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ShipmentRow>> {
    let user = authorize_manage(&state, &headers).await?;
    let old = find_shipment(&state.pool, id).await?;
    if old.shipment_status != "draft" {
        return Err(ApiError::Conflict(
            "only draft shipments can be approved".to_string(),
        ));
    }
    let items = sqlx::query_as::<_, (i64, Decimal, String, Decimal)>(
        "SELECT product_id, quantity, unit, unit_multiplier FROM inventory_shipment_items WHERE shipment_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    let mut tx = state.pool.begin().await?;
    for (product_id, quantity, unit, unit_multiplier) in items {
        let movement = MovementCreate {
            product_id,
            lot_id: None,
            movement_type: "branch_shipment".to_string(),
            quantity,
            unit: Some(unit),
            unit_multiplier: Some(unit_multiplier),
            branch_id: old.branch_id,
            vehicle_id: None,
            reference_table: Some("inventory_shipments".to_string()),
            reference_id: Some(id),
            description: Some("Sube sevkiyati onayi".to_string()),
        };
        apply_movement(&mut tx, &movement, user.id).await?;
    }
    sqlx::query(
        "UPDATE inventory_shipments
         SET shipment_status = 'approved', approver_user_id = $2, shipped_at = COALESCE(shipped_at, now()),
             approved_at = now(), updated_by = $2, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    let row = find_shipment(&state.pool, id).await?;
    audit_update(
        &state,
        &headers,
        "inventory_shipments",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_counts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<CountRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, CountRow>(
        "SELECT c.id, c.count_type, c.count_method, c.category, c.count_status, c.note, c.counted_at,
                COUNT(i.id)::bigint AS item_count, c.created_at, c.updated_at, c.created_by
         FROM inventory_counts c
         LEFT JOIN inventory_count_items i ON i.count_id = c.id
         WHERE c.deleted_at IS NULL
         GROUP BY c.id
         ORDER BY c.created_at DESC
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CountCreate>,
) -> ApiResult<Json<CountRow>> {
    let user = authorize_manage(&state, &headers).await?;
    if payload.items.is_empty() {
        return Err(ApiError::BadRequest(
            "count items cannot be empty".to_string(),
        ));
    }
    let mut tx = state.pool.begin().await?;
    let count_id: i64 = sqlx::query_scalar(
        "INSERT INTO inventory_counts (count_type, count_method, category, note, created_by)
         VALUES ($1, COALESCE($2, 'manual'), $3, $4, $5)
         RETURNING id",
    )
    .bind(payload.count_type)
    .bind(payload.count_method)
    .bind(payload.category)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;
    for item in payload.items {
        let stock: Decimal = sqlx::query_scalar(
            "SELECT current_stock FROM inventory_products WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(item.product_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::NotFound)?;
        let difference = item.counted_stock - stock;
        sqlx::query(
            "INSERT INTO inventory_count_items (count_id, product_id, system_stock, counted_stock, difference, reason)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(count_id)
        .bind(item.product_id)
        .bind(stock)
        .bind(item.counted_stock)
        .bind(difference)
        .bind(item.reason)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let row = find_count(&state.pool, count_id).await?;
    audit_create(&state, &headers, "inventory_counts", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn get_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<CountDetail>> {
    authorize_read(&state, &headers).await?;
    let count = find_count(&state.pool, id).await?;
    let items = list_count_items(&state.pool, id).await?;
    Ok(Json(CountDetail { count, items }))
}

async fn complete_count(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<CountRow>> {
    let user = authorize_manage(&state, &headers).await?;
    let old = find_count(&state.pool, id).await?;
    if old.count_status != "draft" {
        return Err(ApiError::Conflict(
            "only draft counts can be completed".to_string(),
        ));
    }
    let items = sqlx::query_as::<_, (i64, Decimal, Decimal)>(
        "SELECT product_id, counted_stock, difference FROM inventory_count_items WHERE count_id = $1",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    let mut tx = state.pool.begin().await?;
    for (product_id, counted_stock, difference) in items {
        if difference == Decimal::ZERO {
            continue;
        }
        let movement_type = if difference > Decimal::ZERO {
            "manual_in"
        } else {
            "manual_out"
        };
        let movement = MovementCreate {
            product_id,
            lot_id: None,
            movement_type: movement_type.to_string(),
            quantity: difference.abs(),
            unit: Some("adet".to_string()),
            unit_multiplier: Some(Decimal::ONE),
            branch_id: None,
            vehicle_id: None,
            reference_table: Some("inventory_counts".to_string()),
            reference_id: Some(id),
            description: Some(format!("Sayim duzeltmesi. Sayim stok: {}", counted_stock)),
        };
        apply_movement(&mut tx, &movement, user.id).await?;
    }
    sqlx::query("UPDATE inventory_counts SET count_status = 'completed', counted_at = now(), updated_by = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let row = find_count(&state.pool, id).await?;
    audit_update(
        &state,
        &headers,
        "inventory_counts",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_purchase_requests(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<PurchaseRequestRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, PurchaseRequestRow>(
        "SELECT r.id, r.product_id, p.product_name, r.requested_quantity, r.approved_quantity,
                r.request_status, r.reason, r.note, r.created_at, r.updated_at, r.created_by
         FROM inventory_purchase_requests r
         JOIN inventory_products p ON p.id = r.product_id
         WHERE r.deleted_at IS NULL
         ORDER BY
           CASE r.request_status WHEN 'open' THEN 0 WHEN 'approved' THEN 1 WHEN 'ordered' THEN 2 ELSE 3 END,
           r.created_at DESC
         LIMIT 300",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn get_purchase_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<PurchaseRequestRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_purchase_request(&state.pool, id).await?))
}

async fn create_purchase_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PurchaseRequestCreate>,
) -> ApiResult<Json<PurchaseRequestRow>> {
    let user = authorize_request_purchase(&state, &headers).await?;
    if payload.requested_quantity <= Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "requested quantity must be greater than zero".to_string(),
        ));
    }
    find_product(&state.pool, payload.product_id).await?;
    let row = sqlx::query_as::<_, PurchaseRequestRow>(
        "INSERT INTO inventory_purchase_requests
         (product_id, recommended_quantity, requested_quantity, request_status, reason, note, created_by)
         VALUES ($1, $2, $2, 'open', $3, $4, $5)
         RETURNING id, product_id, (SELECT product_name FROM inventory_products WHERE id = product_id) AS product_name,
                   requested_quantity, approved_quantity, request_status, reason, note,
                   created_at, updated_at, created_by",
    )
    .bind(payload.product_id)
    .bind(payload.requested_quantity)
    .bind(payload.reason)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(
        &state,
        &headers,
        "inventory_purchase_requests",
        row.id,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn update_purchase_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<PurchaseRequestUpdate>,
) -> ApiResult<Json<PurchaseRequestRow>> {
    let user = authorize_manage(&state, &headers).await?;
    if let Some(status) = payload.request_status.as_deref() {
        validate_purchase_request_status(status)?;
    }
    if matches!(payload.requested_quantity, Some(value) if value <= Decimal::ZERO) {
        return Err(ApiError::BadRequest(
            "requested quantity must be greater than zero".to_string(),
        ));
    }
    if matches!(payload.approved_quantity, Some(value) if value < Decimal::ZERO) {
        return Err(ApiError::BadRequest(
            "approved quantity cannot be negative".to_string(),
        ));
    }
    let old = find_purchase_request(&state.pool, id).await?;
    let row = sqlx::query_as::<_, PurchaseRequestRow>(
        "UPDATE inventory_purchase_requests SET
            requested_quantity = COALESCE($2, requested_quantity),
            approved_quantity = COALESCE($3, approved_quantity),
            request_status = COALESCE($4, request_status),
            note = COALESCE($5, note),
            updated_by = $6,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, product_id, (SELECT product_name FROM inventory_products WHERE id = product_id) AS product_name,
                   requested_quantity, approved_quantity, request_status, reason, note,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.requested_quantity)
    .bind(payload.approved_quantity)
    .bind(payload.request_status)
    .bind(payload.note)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(
        &state,
        &headers,
        "inventory_purchase_requests",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn get_inventory_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<InventoryDashboard>> {
    authorize_read(&state, &headers).await?;
    let summary = sqlx::query_as::<_, InventoryDashboardSummary>(
        "SELECT
            (SELECT COUNT(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true)::bigint AS total_products,
            (SELECT SUM(current_stock * COALESCE(unit_cost, 0)) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true) AS total_stock_value,
            (SELECT COUNT(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true AND current_stock > 0 AND current_stock <= COALESCE(critical_stock, minimum_stock))::bigint AS critical_stock_count,
            (SELECT COUNT(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true AND current_stock <= 0)::bigint AS out_of_stock_count,
            (SELECT COUNT(*) FROM inventory_shipments WHERE deleted_at IS NULL AND shipment_status = 'draft')::bigint AS pending_shipments,
            (SELECT COUNT(*) FROM inventory_purchase_requests WHERE deleted_at IS NULL AND request_status = 'open')::bigint AS pending_purchase_requests,
            (SELECT COUNT(*) FROM inventory_lots WHERE deleted_at IS NULL AND expiry_date IS NOT NULL AND expiry_date <= CURRENT_DATE + interval '30 days')::bigint AS expiring_lots_30_days",
    )
    .fetch_one(&state.pool)
    .await?;
    let recent_movements = query_movements(
        &state.pool,
        MovementQuery {
            product_id: None,
            movement_type: None,
            branch_id: None,
            vehicle_id: None,
            date_from: None,
            date_to: None,
            limit: Some(6),
        },
    )
    .await?;
    let recent_notifications = sqlx::query_as::<_, InventoryNotificationRow>(
        "SELECT id, notification_type, message, sent_via::text AS sent_via, delivery_status, created_at, read_at
         FROM notifications
         WHERE notification_type LIKE 'inventory_%' OR message ILIKE '%stok%'
         ORDER BY created_at DESC
         LIMIT 6",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(InventoryDashboard {
        total_products: summary.total_products,
        total_stock_value: summary.total_stock_value,
        critical_stock_count: summary.critical_stock_count,
        out_of_stock_count: summary.out_of_stock_count,
        pending_shipments: summary.pending_shipments,
        pending_purchase_requests: summary.pending_purchase_requests,
        expiring_lots_30_days: summary.expiring_lots_30_days,
        recent_movements,
        recent_notifications,
    }))
}

async fn list_stock_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<StockAlertRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, StockAlertRow>(
        "SELECT id AS product_id, product_code, product_name, current_stock, minimum_stock, critical_stock,
                CASE
                    WHEN current_stock <= 0 THEN 'red'
                    WHEN current_stock < minimum_stock THEN 'orange'
                    ELSE 'yellow'
                END AS alert_level,
                GREATEST(COALESCE(maximum_stock, minimum_stock * 2) - current_stock, 0) AS suggested_order_quantity
         FROM inventory_products
         WHERE deleted_at IS NULL AND is_active = true
           AND current_stock <= GREATEST(minimum_stock, COALESCE(critical_stock, minimum_stock))
         ORDER BY current_stock ASC, product_name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn list_purchase_suggestions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<PurchaseSuggestionRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, PurchaseSuggestionRow>(
        "WITH consumption AS (
            SELECT product_id,
                   ABS(SUM(CASE WHEN movement_at >= now() - interval '3 months' THEN LEAST(base_quantity, 0) ELSE 0 END)) / 3 AS m3,
                   ABS(SUM(CASE WHEN movement_at >= now() - interval '6 months' THEN LEAST(base_quantity, 0) ELSE 0 END)) / 6 AS m6,
                   ABS(SUM(CASE WHEN movement_at >= now() - interval '12 months' THEN LEAST(base_quantity, 0) ELSE 0 END)) / 12 AS m12
            FROM inventory_movements
            WHERE cancelled_at IS NULL
            GROUP BY product_id
         )
         SELECT p.id AS product_id, p.product_code, p.product_name, p.current_stock,
                COALESCE(c.m3, 0) AS monthly_consumption_3m,
                COALESCE(c.m6, 0) AS monthly_consumption_6m,
                COALESCE(c.m12, 0) AS monthly_consumption_12m,
                ((COALESCE(c.m3, 0) + COALESCE(c.m6, 0) + COALESCE(c.m12, 0)) / 3) AS average_monthly_consumption,
                CASE WHEN ((COALESCE(c.m3, 0) + COALESCE(c.m6, 0) + COALESCE(c.m12, 0)) / 3) > 0
                     THEN (CURRENT_DATE + CEIL(p.current_stock / ((COALESCE(c.m3, 0) + COALESCE(c.m6, 0) + COALESCE(c.m12, 0)) / 3) * 30)::int)
                     ELSE NULL END AS estimated_runout_date,
                GREATEST(COALESCE(p.maximum_stock, p.minimum_stock * 2) - p.current_stock, 0) AS suggested_order_quantity
         FROM inventory_products p
         LEFT JOIN consumption c ON c.product_id = p.id
         WHERE p.deleted_at IS NULL AND p.is_active = true
           AND p.current_stock <= GREATEST(p.minimum_stock, COALESCE(p.critical_stock, p.minimum_stock))
         ORDER BY suggested_order_quantity DESC, p.product_name",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn apply_movement(
    tx: &mut Transaction<'_, Postgres>,
    payload: &MovementCreate,
    actor_id: i64,
) -> ApiResult<MovementRow> {
    validate_movement_type(&payload.movement_type)?;
    if payload.quantity <= Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "quantity must be positive".to_string(),
        ));
    }
    let multiplier = payload.unit_multiplier.unwrap_or(Decimal::ONE);
    if multiplier <= Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "unit multiplier must be positive".to_string(),
        ));
    }
    let base_quantity = payload.quantity * multiplier;
    let delta = stock_delta(&payload.movement_type, base_quantity);
    let previous_stock: Decimal = sqlx::query_scalar(
        "SELECT current_stock FROM inventory_products WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(payload.product_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(ApiError::NotFound)?;
    let next_stock = previous_stock + delta;
    if next_stock < Decimal::ZERO {
        return Err(ApiError::Conflict("stock cannot go below zero".to_string()));
    }
    sqlx::query(
        "UPDATE inventory_products
         SET current_stock = $2,
             available_stock = GREATEST($2 - reserved_stock, 0),
             updated_by = $3,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(payload.product_id)
    .bind(next_stock)
    .bind(actor_id)
    .execute(&mut **tx)
    .await?;
    if let Some(lot_id) = payload.lot_id {
        let (lot_product_id, lot_quantity) = sqlx::query_as::<_, (i64, Decimal)>(
            "SELECT product_id, quantity
             FROM inventory_lots
             WHERE id = $1 AND deleted_at IS NULL
             FOR UPDATE",
        )
        .bind(lot_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(ApiError::NotFound)?;

        if lot_product_id != payload.product_id {
            return Err(ApiError::BadRequest(
                "lot does not belong to selected product".to_string(),
            ));
        }

        let next_lot_quantity = lot_quantity + delta;
        if next_lot_quantity < Decimal::ZERO {
            return Err(ApiError::Conflict(
                "lot stock cannot go below zero".to_string(),
            ));
        }

        sqlx::query(
            "UPDATE inventory_lots
             SET quantity = $2, updated_by = $3, updated_at = now()
             WHERE id = $1",
        )
        .bind(lot_id)
        .bind(next_lot_quantity)
        .bind(actor_id)
        .execute(&mut **tx)
        .await?;
    }
    sqlx::query_as::<_, MovementRow>(
        "INSERT INTO inventory_movements
         (product_id, lot_id, movement_type, quantity, unit, unit_multiplier, base_quantity,
          previous_stock, next_stock, branch_id, vehicle_id, reference_table, reference_id, description, created_by)
         VALUES ($1, $2, $3, $4, COALESCE($5, 'adet'), $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
         RETURNING id, product_id,
                   (SELECT product_name FROM inventory_products WHERE id = product_id) AS product_name,
                   (SELECT product_code FROM inventory_products WHERE id = product_id) AS product_code,
                   lot_id,
                   (SELECT lot_number FROM inventory_lots WHERE id = lot_id) AS lot_number,
                   movement_type, quantity, unit, unit_multiplier, base_quantity,
                   previous_stock, next_stock, branch_id,
                   (SELECT name FROM departments WHERE id = branch_id) AS branch_name,
                   vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   reference_table, reference_id, description, movement_at, cancelled_at, cancel_reason,
                   created_at, created_by",
    )
    .bind(payload.product_id)
    .bind(payload.lot_id)
    .bind(&payload.movement_type)
    .bind(payload.quantity)
    .bind(payload.unit.clone())
    .bind(multiplier)
    .bind(delta)
    .bind(previous_stock)
    .bind(next_stock)
    .bind(payload.branch_id)
    .bind(payload.vehicle_id)
    .bind(payload.reference_table.clone())
    .bind(payload.reference_id)
    .bind(payload.description.clone())
    .bind(actor_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(ApiError::from)
}

fn stock_delta(movement_type: &str, quantity: Decimal) -> Decimal {
    match movement_type {
        "warehouse_in" | "branch_return" | "count_adjustment" | "manual_in" => quantity,
        "warehouse_out" | "branch_shipment" | "scrap_out" | "vehicle_usage" | "manual_out" => {
            -quantity
        }
        _ => Decimal::ZERO,
    }
}

fn validate_movement_type(movement_type: &str) -> ApiResult<()> {
    match movement_type {
        "warehouse_in" | "warehouse_out" | "branch_shipment" | "branch_return"
        | "count_adjustment" | "scrap_out" | "vehicle_usage" | "manual_in" | "manual_out" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid inventory movement type".to_string(),
        )),
    }
}

fn validate_unit(unit: &str) -> ApiResult<()> {
    match unit {
        "adet" | "koli" | "paket" | "kutu" | "kg" | "litre" | "metre" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid inventory unit".to_string())),
    }
}

async fn query_movements(pool: &PgPool, query: MovementQuery) -> ApiResult<Vec<MovementRow>> {
    Ok(sqlx::query_as::<_, MovementRow>(
        "SELECT m.id, m.product_id, p.product_name, p.product_code, m.lot_id, l.lot_number, m.movement_type,
                m.quantity, m.unit, m.unit_multiplier, m.base_quantity, m.previous_stock, m.next_stock,
                m.branch_id, d.name AS branch_name, m.vehicle_id, v.plate, m.reference_table, m.reference_id,
                m.description, m.movement_at, m.cancelled_at, m.cancel_reason, m.created_at, m.created_by
         FROM inventory_movements m
         JOIN inventory_products p ON p.id = m.product_id
         LEFT JOIN inventory_lots l ON l.id = m.lot_id
         LEFT JOIN departments d ON d.id = m.branch_id
         LEFT JOIN vehicles v ON v.id = m.vehicle_id
         WHERE ($1::bigint IS NULL OR m.product_id = $1)
           AND ($2::text IS NULL OR m.movement_type = $2)
           AND ($3::bigint IS NULL OR m.branch_id = $3)
           AND ($4::bigint IS NULL OR m.vehicle_id = $4)
           AND ($5::date IS NULL OR m.movement_at::date >= $5)
           AND ($6::date IS NULL OR m.movement_at::date <= $6)
         ORDER BY m.movement_at DESC
         LIMIT $7",
    )
    .bind(query.product_id)
    .bind(query.movement_type)
    .bind(query.branch_id)
    .bind(query.vehicle_id)
    .bind(query.date_from)
    .bind(query.date_to)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(pool)
    .await?)
}

async fn find_product(pool: &PgPool, id: i64) -> ApiResult<ProductRow> {
    sqlx::query_as::<_, ProductRow>(
        "SELECT id, product_code, barcode, qr_code, product_name, category, sub_category, brand,
                description, main_unit, package_unit, package_multiplier, current_stock,
                available_stock, reserved_stock, minimum_stock, maximum_stock, critical_stock,
                safety_stock, unit_cost, is_lot_tracked, expiry_tracking, is_active,
                created_at, updated_at, created_by
         FROM inventory_products
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_movement(pool: &PgPool, id: i64) -> ApiResult<MovementRow> {
    sqlx::query_as::<_, MovementRow>(
        "SELECT m.id, m.product_id, p.product_name, p.product_code, m.lot_id, l.lot_number, m.movement_type,
                m.quantity, m.unit, m.unit_multiplier, m.base_quantity, m.previous_stock, m.next_stock,
                m.branch_id, d.name AS branch_name, m.vehicle_id, v.plate, m.reference_table, m.reference_id,
                m.description, m.movement_at, m.cancelled_at, m.cancel_reason, m.created_at, m.created_by
         FROM inventory_movements m
         JOIN inventory_products p ON p.id = m.product_id
         LEFT JOIN inventory_lots l ON l.id = m.lot_id
         LEFT JOIN departments d ON d.id = m.branch_id
         LEFT JOIN vehicles v ON v.id = m.vehicle_id
         WHERE m.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_shipment(pool: &PgPool, id: i64) -> ApiResult<ShipmentRow> {
    sqlx::query_as::<_, ShipmentRow>(
        "SELECT s.id, s.branch_id, d.name AS branch_name, s.shipment_status, s.sender_user_id,
                s.approver_user_id, s.shipped_at, s.approved_at, s.note,
                COUNT(i.id)::bigint AS item_count, s.created_at, s.updated_at, s.created_by
         FROM inventory_shipments s
         LEFT JOIN departments d ON d.id = s.branch_id
         LEFT JOIN inventory_shipment_items i ON i.shipment_id = s.id
         WHERE s.id = $1 AND s.deleted_at IS NULL
         GROUP BY s.id, d.name",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn list_shipment_items(pool: &PgPool, shipment_id: i64) -> ApiResult<Vec<ShipmentItemRow>> {
    Ok(sqlx::query_as::<_, ShipmentItemRow>(
        "SELECT i.id, i.shipment_id, i.product_id, p.product_name,
                i.quantity, i.unit, i.unit_multiplier, i.base_quantity
         FROM inventory_shipment_items i
         JOIN inventory_products p ON p.id = i.product_id
         WHERE i.shipment_id = $1
         ORDER BY i.id ASC",
    )
    .bind(shipment_id)
    .fetch_all(pool)
    .await?)
}

async fn find_count(pool: &PgPool, id: i64) -> ApiResult<CountRow> {
    sqlx::query_as::<_, CountRow>(
        "SELECT c.id, c.count_type, c.count_method, c.category, c.count_status, c.note, c.counted_at,
                COUNT(i.id)::bigint AS item_count, c.created_at, c.updated_at, c.created_by
         FROM inventory_counts c
         LEFT JOIN inventory_count_items i ON i.count_id = c.id
         WHERE c.id = $1 AND c.deleted_at IS NULL
         GROUP BY c.id",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn list_count_items(pool: &PgPool, count_id: i64) -> ApiResult<Vec<CountItemRow>> {
    Ok(sqlx::query_as::<_, CountItemRow>(
        "SELECT i.id, i.count_id, i.product_id, p.product_name,
                i.system_stock, i.counted_stock, i.difference, i.reason
         FROM inventory_count_items i
         JOIN inventory_products p ON p.id = i.product_id
         WHERE i.count_id = $1
         ORDER BY i.id ASC",
    )
    .bind(count_id)
    .fetch_all(pool)
    .await?)
}

async fn find_purchase_request(pool: &PgPool, id: i64) -> ApiResult<PurchaseRequestRow> {
    sqlx::query_as::<_, PurchaseRequestRow>(
        "SELECT r.id, r.product_id, p.product_name, r.requested_quantity, r.approved_quantity,
                r.request_status, r.reason, r.note, r.created_at, r.updated_at, r.created_by
         FROM inventory_purchase_requests r
         JOIN inventory_products p ON p.id = r.product_id
         WHERE r.id = $1 AND r.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn authorize_read(state: &AppState, headers: &HeaderMap) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles_or_mobile_permission(
        &state.pool,
        &user,
        &["admin", "manager", "operation", "accounting"],
        "inventory_view",
        "view",
    )
    .await?;
    Ok(user)
}

fn validate_purchase_request_status(status: &str) -> ApiResult<()> {
    match status {
        "open" | "approved" | "ordered" | "received" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid purchase request status".to_string(),
        )),
    }
}

async fn authorize_move(state: &AppState, headers: &HeaderMap) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles_or_mobile_permission(
        &state.pool,
        &user,
        &["admin", "manager", "operation"],
        "inventory_view",
        "create",
    )
    .await?;
    Ok(user)
}

async fn authorize_manage(state: &AppState, headers: &HeaderMap) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles(&user, &["admin", "manager"])?;
    Ok(user)
}

async fn authorize_request_purchase(
    state: &AppState,
    headers: &HeaderMap,
) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles_or_mobile_permission(
        &state.pool,
        &user,
        &["admin", "manager", "operation", "accounting"],
        "inventory_order",
        "create",
    )
    .await?;
    Ok(user)
}

async fn create_stock_notification_if_needed(
    pool: &PgPool,
    movement: &MovementRow,
    actor_id: i64,
) -> ApiResult<()> {
    let alert = sqlx::query_as::<_, StockAlertRow>(
        "SELECT id AS product_id, product_code, product_name, current_stock, minimum_stock, critical_stock,
                CASE WHEN current_stock <= 0 THEN 'red'
                     WHEN current_stock < minimum_stock THEN 'orange'
                     ELSE 'yellow' END AS alert_level,
                GREATEST(COALESCE(maximum_stock, minimum_stock * 2) - current_stock, 0) AS suggested_order_quantity
         FROM inventory_products
         WHERE id = $1 AND deleted_at IS NULL AND is_active = true
           AND current_stock <= GREATEST(minimum_stock, COALESCE(critical_stock, minimum_stock))",
    )
    .bind(movement.product_id)
    .fetch_optional(pool)
    .await?;

    if let Some(alert) = alert {
        sqlx::query(
            "INSERT INTO notifications
             (notification_type, related_vehicle_id, message, sent_via, delivery_status, created_by)
             VALUES ('inventory_stock_alert', NULL, $1, 'system', 'pending', $2)",
        )
        .bind(format!(
            "Stok alarmi: {} mevcut {}, minimum {}, seviye {}",
            alert.product_name, alert.current_stock, alert.minimum_stock, alert.alert_level
        ))
        .bind(actor_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn audit_create<T: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    row: &T,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "create",
        None,
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}

async fn audit_update<T: Serialize, U: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    old: &T,
    row: &U,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "update",
        Some(serde_json::to_value(old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}

async fn audit_delete<T: Serialize, U: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    old: &T,
    row: &U,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "delete",
        Some(serde_json::to_value(old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}

fn map_product_unique_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.constraint() == Some("inventory_products_product_code_key") {
            return ApiError::Conflict("product code already exists".to_string());
        }
        if db_err.constraint() == Some("inventory_products_barcode_key") {
            return ApiError::Conflict("barcode already exists".to_string());
        }
    }
    ApiError::Database(err)
}
