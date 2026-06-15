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
use sqlx::{FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_vehicles).post(create_vehicle))
        .route(
            "/:id",
            get(get_vehicle)
                .patch(update_vehicle)
                .delete(soft_delete_vehicle),
        )
        .route("/:id/profile", get(get_vehicle_profile))
        .route("/:id/sell", post(sell_vehicle))
        .route("/sold/:id", get(get_sold_vehicle_detail))
}

#[derive(Debug, Deserialize)]
struct VehicleQuery {
    status: Option<String>,
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VehicleCreate {
    plate: String,
    brand: String,
    model: String,
    model_year: Option<i32>,
    vehicle_type: Option<String>,
    fuel_type: Option<String>,
    transmission: Option<String>,
    chassis_no: Option<String>,
    engine_no: Option<String>,
    warranty_status: Option<String>,
    warranty_end: Option<NaiveDate>,
    has_hgs: Option<bool>,
    has_mobiliz: Option<bool>,
    has_kopilot: Option<bool>,
    has_k2: Option<bool>,
    tasitmatik_company: Option<String>,
    spare_key_location: Option<String>,
    company_id: Option<i64>,
    department_id: Option<i64>,
    user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct VehicleUpdate {
    brand: Option<String>,
    model: Option<String>,
    model_year: Option<i32>,
    vehicle_type: Option<String>,
    fuel_type: Option<String>,
    transmission: Option<String>,
    chassis_no: Option<String>,
    engine_no: Option<String>,
    warranty_status: Option<String>,
    warranty_end: Option<NaiveDate>,
    has_hgs: Option<bool>,
    has_mobiliz: Option<bool>,
    has_kopilot: Option<bool>,
    has_k2: Option<bool>,
    tasitmatik_company: Option<String>,
    spare_key_location: Option<String>,
    company_id: Option<i64>,
    department_id: Option<i64>,
    user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SellVehicleRequest {
    sold_date: NaiveDate,
    sold_reason: String,
    sold_price: Option<Decimal>,
    buyer_info: Option<String>,
    company_exit_reason: String,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleRow {
    id: i64,
    plate: String,
    brand: String,
    model: String,
    model_year: Option<i32>,
    vehicle_type: Option<String>,
    fuel_type: Option<String>,
    transmission: Option<String>,
    chassis_no: Option<String>,
    engine_no: Option<String>,
    warranty_status: Option<String>,
    warranty_end: Option<NaiveDate>,
    has_hgs: bool,
    has_mobiliz: bool,
    has_kopilot: bool,
    has_k2: bool,
    tasitmatik_company: Option<String>,
    spare_key_location: Option<String>,
    status: String,
    company_id: Option<i64>,
    department_id: Option<i64>,
    user_id: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
}

#[derive(Debug, FromRow, Serialize)]
struct SoldVehicleRow {
    id: i64,
    vehicle_id: i64,
    sold_date: NaiveDate,
    sold_reason: String,
    sold_price: Option<Decimal>,
    buyer_info: Option<String>,
    company_exit_reason: String,
    note: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct SoldVehicleDetail {
    sold: SoldVehicleRow,
    vehicle: VehicleRow,
    km_summary: KmSummary,
    maintenance_summary: MaintenanceSummary,
    policy_summary: PolicySummary,
    expense_summary: ExpenseSummary,
    damage_summary: DamageSummary,
    file_summary: SoldVehicleFileSummary,
}

#[derive(Debug, Serialize)]
struct VehicleProfile {
    vehicle: VehicleRow,
    km_summary: KmSummary,
    maintenance_summary: MaintenanceSummary,
    policy_summary: PolicySummary,
    expense_summary: ExpenseSummary,
    damage_summary: DamageSummary,
    operations_summary: OperationsSummary,
    file_summary: VehicleFileSummary,
    recent_tasks: Vec<VehicleTaskRow>,
    recent_km_logs: Vec<VehicleKmLogRow>,
}

#[derive(Debug, FromRow, Serialize)]
struct KmSummary {
    total_logs: i64,
    last_km: Option<i32>,
    suspicious_logs: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct MaintenanceSummary {
    total_records: i64,
    completed_records: i64,
    total_cost: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct PolicySummary {
    total_policies: i64,
    active_policies: i64,
    latest_end_date: Option<NaiveDate>,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseSummary {
    total_records: i64,
    total_amount: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct DamageSummary {
    total_records: i64,
    open_records: i64,
    total_estimated_cost: Option<Decimal>,
    total_actual_cost: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct SoldVehicleFileSummary {
    total_files: i64,
    vehicle_files: i64,
    sale_documents: i64,
    transfer_documents: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct OperationsSummary {
    inspection_count: i64,
    open_inspection_count: i64,
    value_loss_count: i64,
    open_value_loss_count: i64,
    fuel_record_count: i64,
    wash_count: i64,
    quote_count: i64,
    pending_quote_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleFileSummary {
    total_files: i64,
    vehicle_files: i64,
    expense_files: i64,
    damage_files: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleTaskRow {
    id: i64,
    task_type: String,
    priority: String,
    due_date: Option<NaiveDate>,
    task_status: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleKmLogRow {
    id: i64,
    km: i32,
    entry_type: String,
    verification_status: String,
    created_at: DateTime<Utc>,
}

async fn list_vehicles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<VehicleQuery>,
) -> ApiResult<Json<Vec<VehicleRow>>> {
    let user = auth::current_user(&state, &headers).await?;
    let status = query.status.unwrap_or_else(|| "active".to_string());
    let q = query.q.map(|value| format!("%{}%", value.to_uppercase()));
    let has_role_scope = matches!(
        user.role.as_str(),
        "admin" | "manager" | "operation" | "accounting"
    );
    let has_mobile_scope =
        auth::has_mobile_permission(&state.pool, &user, "vehicle_view", "view").await?;

    let rows = sqlx::query_as::<_, VehicleRow>(
        "SELECT id, plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
                chassis_no, engine_no, warranty_status, warranty_end,
                has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
                status::text AS status, company_id, department_id, user_id, created_at, updated_at, is_active
         FROM vehicles
         WHERE deleted_at IS NULL
           AND ($1 = 'all' OR status::text = $1)
           AND ($2::text IS NULL OR plate ILIKE $2)
           AND (
             $3::boolean = true
             OR user_id = $4
             OR (
               $5::boolean = true
               AND ($6::bigint IS NULL OR company_id = $6)
               AND ($7::bigint IS NULL OR department_id = $7)
             )
           )
         ORDER BY plate",
    )
    .bind(status)
    .bind(q)
    .bind(has_role_scope)
    .bind(user.id)
    .bind(has_mobile_scope)
    .bind(user.company_id)
    .bind(user.department_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VehicleCreate>,
) -> ApiResult<Json<VehicleRow>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation"])?;

    let plate = payload.plate.trim().to_uppercase();
    let row = sqlx::query_as::<_, VehicleRow>(
        "INSERT INTO vehicles
         (plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
          chassis_no, engine_no, warranty_status, warranty_end,
          has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
          company_id, department_id, user_id, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                 COALESCE($12, false), COALESCE($13, false), COALESCE($14, false), COALESCE($15, false),
                 $16, $17, $18, $19, $20, $21)
         RETURNING id, plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
                   chassis_no, engine_no, warranty_status, warranty_end,
                   has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
                   status::text AS status, company_id, department_id, user_id, created_at, updated_at, is_active",
    )
    .bind(plate)
    .bind(payload.brand)
    .bind(payload.model)
    .bind(payload.model_year)
    .bind(payload.vehicle_type)
    .bind(payload.fuel_type)
    .bind(payload.transmission)
    .bind(payload.chassis_no)
    .bind(payload.engine_no)
    .bind(payload.warranty_status)
    .bind(payload.warranty_end)
    .bind(payload.has_hgs)
    .bind(payload.has_mobiliz)
    .bind(payload.has_kopilot)
    .bind(payload.has_k2)
    .bind(payload.tasitmatik_company)
    .bind(payload.spare_key_location)
    .bind(payload.company_id)
    .bind(payload.department_id)
    .bind(payload.user_id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_unique_vehicle_error)?;

    audit::write_audit(
        &state.pool,
        "vehicles",
        Some(row.id),
        "create",
        None,
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(user.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn get_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleRow>> {
    let user = auth::current_user(&state, &headers).await?;
    let row = find_vehicle(&state.pool, id).await?;
    if !matches!(
        user.role.as_str(),
        "admin" | "manager" | "operation" | "accounting"
    ) && row.user_id != Some(user.id)
    {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(row))
}

async fn get_vehicle_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleProfile>> {
    let user = auth::current_user(&state, &headers).await?;
    let vehicle = find_vehicle(&state.pool, id).await?;
    if !matches!(
        user.role.as_str(),
        "admin" | "manager" | "operation" | "accounting"
    ) && vehicle.user_id != Some(user.id)
    {
        return Err(ApiError::Forbidden);
    }

    Ok(Json(VehicleProfile {
        km_summary: km_summary(&state.pool, id).await?,
        maintenance_summary: maintenance_summary(&state.pool, id).await?,
        policy_summary: policy_summary(&state.pool, id).await?,
        expense_summary: expense_summary(&state.pool, id).await?,
        damage_summary: damage_summary(&state.pool, id).await?,
        operations_summary: operations_summary(&state.pool, id).await?,
        file_summary: vehicle_file_summary(&state.pool, id).await?,
        recent_tasks: recent_vehicle_tasks(&state.pool, id).await?,
        recent_km_logs: recent_vehicle_km_logs(&state.pool, id).await?,
        vehicle,
    }))
}

async fn update_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<VehicleUpdate>,
) -> ApiResult<Json<VehicleRow>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation"])?;
    let old = find_vehicle(&state.pool, id).await?;
    if old.status == "sold" {
        return Err(ApiError::Conflict(
            "sold vehicles cannot be updated".to_string(),
        ));
    }

    let row = sqlx::query_as::<_, VehicleRow>(
        "UPDATE vehicles SET
            brand = COALESCE($2, brand),
            model = COALESCE($3, model),
            model_year = COALESCE($4, model_year),
            vehicle_type = COALESCE($5, vehicle_type),
            fuel_type = COALESCE($6, fuel_type),
            transmission = COALESCE($7, transmission),
            chassis_no = COALESCE($8, chassis_no),
            engine_no = COALESCE($9, engine_no),
            warranty_status = COALESCE($10, warranty_status),
            warranty_end = COALESCE($11, warranty_end),
            has_hgs = COALESCE($12, has_hgs),
            has_mobiliz = COALESCE($13, has_mobiliz),
            has_kopilot = COALESCE($14, has_kopilot),
            has_k2 = COALESCE($15, has_k2),
            tasitmatik_company = COALESCE($16, tasitmatik_company),
            spare_key_location = COALESCE($17, spare_key_location),
            company_id = COALESCE($18, company_id),
            department_id = COALESCE($19, department_id),
            user_id = COALESCE($20, user_id),
            updated_by = $21,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
                   chassis_no, engine_no, warranty_status, warranty_end,
                   has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
                   status::text AS status, company_id, department_id, user_id, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(payload.brand)
    .bind(payload.model)
    .bind(payload.model_year)
    .bind(payload.vehicle_type)
    .bind(payload.fuel_type)
    .bind(payload.transmission)
    .bind(payload.chassis_no)
    .bind(payload.engine_no)
    .bind(payload.warranty_status)
    .bind(payload.warranty_end)
    .bind(payload.has_hgs)
    .bind(payload.has_mobiliz)
    .bind(payload.has_kopilot)
    .bind(payload.has_k2)
    .bind(payload.tasitmatik_company)
    .bind(payload.spare_key_location)
    .bind(payload.company_id)
    .bind(payload.department_id)
    .bind(payload.user_id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "vehicles",
        Some(row.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(user.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn soft_delete_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleRow>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager"])?;
    let old = find_vehicle(&state.pool, id).await?;

    let row = sqlx::query_as::<_, VehicleRow>(
        "UPDATE vehicles SET status = 'passive', is_active = false, deleted_at = now(),
             updated_by = $2, updated_at = now()
         WHERE id = $1
         RETURNING id, plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
                   chassis_no, engine_no, warranty_status, warranty_end,
                   has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
                   status::text AS status, company_id, department_id, user_id, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "vehicles",
        Some(row.id),
        "delete",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(user.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn sell_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<SellVehicleRequest>,
) -> ApiResult<Json<SoldVehicleRow>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager"])?;
    let old = find_vehicle(&state.pool, id).await?;
    if old.status == "sold" {
        return Err(ApiError::Conflict("vehicle already sold".to_string()));
    }

    let mut tx = state.pool.begin().await?;

    sqlx::query(
        "UPDATE vehicles SET status = 'sold', is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE tasks SET task_status = 'cancelled', updated_by = $2, updated_at = now()
         WHERE related_vehicle_id = $1 AND task_status NOT IN ('completed', 'cancelled')",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE vehicle_assignments
         SET released_at = COALESCE(released_at, now()),
             updated_by = $2,
             updated_at = now()
         WHERE vehicle_id = $1 AND released_at IS NULL AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    let sold = sqlx::query_as::<_, SoldVehicleRow>(
        "INSERT INTO sold_vehicles
         (vehicle_id, sold_date, sold_reason, sold_price, buyer_info, company_exit_reason, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, vehicle_id, sold_date, sold_reason, sold_price, buyer_info,
                   company_exit_reason, note, created_at",
    )
    .bind(id)
    .bind(payload.sold_date)
    .bind(payload.sold_reason)
    .bind(payload.sold_price)
    .bind(payload.buyer_info)
    .bind(payload.company_exit_reason)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::write_audit(
        &state.pool,
        "vehicles",
        Some(id),
        "sell",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        json!({ "sold_vehicle_id": sold.id }),
        Some(user.id),
        &headers,
    )
    .await?;

    Ok(Json(sold))
}

async fn get_sold_vehicle_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<SoldVehicleDetail>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation", "accounting"])?;

    let sold = sqlx::query_as::<_, SoldVehicleRow>(
        "SELECT id, vehicle_id, sold_date, sold_reason, sold_price, buyer_info,
                company_exit_reason, note, created_at
         FROM sold_vehicles WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let vehicle = find_vehicle_include_sold(&state.pool, sold.vehicle_id).await?;
    let sold_vehicle_id = sold.id;
    let vehicle_id = sold.vehicle_id;
    let detail = SoldVehicleDetail {
        km_summary: km_summary(&state.pool, vehicle_id).await?,
        maintenance_summary: maintenance_summary(&state.pool, vehicle_id).await?,
        policy_summary: policy_summary(&state.pool, vehicle_id).await?,
        expense_summary: expense_summary(&state.pool, vehicle_id).await?,
        damage_summary: damage_summary(&state.pool, vehicle_id).await?,
        file_summary: sold_vehicle_file_summary(&state.pool, sold_vehicle_id, vehicle_id).await?,
        sold,
        vehicle,
    };

    Ok(Json(detail))
}

async fn find_vehicle(pool: &PgPool, id: i64) -> ApiResult<VehicleRow> {
    let row = find_vehicle_include_sold(pool, id).await?;
    if row.status == "sold" || !row.is_active {
        return Err(ApiError::NotFound);
    }
    Ok(row)
}

async fn find_vehicle_include_sold(pool: &PgPool, id: i64) -> ApiResult<VehicleRow> {
    sqlx::query_as::<_, VehicleRow>(
        "SELECT id, plate, brand, model, model_year, vehicle_type, fuel_type, transmission,
                chassis_no, engine_no, warranty_status, warranty_end,
                has_hgs, has_mobiliz, has_kopilot, has_k2, tasitmatik_company, spare_key_location,
                status::text AS status, company_id, department_id, user_id, created_at, updated_at, is_active
         FROM vehicles WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn km_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<KmSummary> {
    Ok(sqlx::query_as::<_, KmSummary>(
        "SELECT
            count(*) AS total_logs,
            max(km) AS last_km,
            count(*) FILTER (WHERE verification_status = 'suspicious'::verification_status) AS suspicious_logs
         FROM km_logs
         WHERE vehicle_id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn maintenance_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<MaintenanceSummary> {
    Ok(sqlx::query_as::<_, MaintenanceSummary>(
        "SELECT
            count(*) AS total_records,
            count(*) FILTER (WHERE maintenance_status = 'completed'::maintenance_status) AS completed_records,
            sum(total_cost) AS total_cost
         FROM maintenances
         WHERE vehicle_id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn policy_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<PolicySummary> {
    Ok(sqlx::query_as::<_, PolicySummary>(
        "SELECT
            count(*) AS total_policies,
            count(*) FILTER (WHERE renewal_status = 'active'::renewal_status) AS active_policies,
            max(end_date) AS latest_end_date
         FROM insurance_policies
         WHERE vehicle_id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn expense_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<ExpenseSummary> {
    Ok(sqlx::query_as::<_, ExpenseSummary>(
        "SELECT
            count(*) AS total_records,
            sum(amount) AS total_amount
         FROM expenses
         WHERE vehicle_id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn damage_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<DamageSummary> {
    Ok(sqlx::query_as::<_, DamageSummary>(
        "SELECT
            count(*) AS total_records,
            count(*) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')) AS open_records,
            sum(estimated_cost) AS total_estimated_cost,
            sum(actual_cost) AS total_actual_cost
         FROM damages
         WHERE vehicle_id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn operations_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<OperationsSummary> {
    Ok(sqlx::query_as::<_, OperationsSummary>(
        "SELECT
            (SELECT count(*) FROM vehicle_inspections WHERE vehicle_id = $1 AND deleted_at IS NULL) AS inspection_count,
            (SELECT count(*) FROM vehicle_inspections WHERE vehicle_id = $1 AND deleted_at IS NULL AND inspection_status NOT IN ('completed', 'closed', 'cancelled')) AS open_inspection_count,
            (SELECT count(*) FROM value_loss_claims WHERE vehicle_id = $1 AND deleted_at IS NULL) AS value_loss_count,
            (SELECT count(*) FROM value_loss_claims WHERE vehicle_id = $1 AND deleted_at IS NULL AND claim_status NOT IN ('closed', 'cancelled', 'paid')) AS open_value_loss_count,
            (SELECT count(*) FROM fuel_entries WHERE vehicle_id = $1 AND deleted_at IS NULL) AS fuel_record_count,
            (SELECT count(*) FROM vehicle_washes WHERE vehicle_id = $1 AND deleted_at IS NULL) AS wash_count,
            (SELECT count(*) FROM insurance_quotes WHERE vehicle_id = $1 AND deleted_at IS NULL) AS quote_count,
            (SELECT count(*) FROM insurance_quotes WHERE vehicle_id = $1 AND deleted_at IS NULL AND quote_status = 'pending') AS pending_quote_count",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn vehicle_file_summary(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleFileSummary> {
    Ok(sqlx::query_as::<_, VehicleFileSummary>(
        "SELECT
            count(*) AS total_files,
            count(*) FILTER (WHERE module_name = 'vehicles') AS vehicle_files,
            count(*) FILTER (WHERE module_name = 'expenses') AS expense_files,
            count(*) FILTER (WHERE module_name = 'damages') AS damage_files
         FROM file_documents
         WHERE deleted_at IS NULL
           AND (
                (module_name = 'vehicles' AND entity_id = $1)
                OR (module_name = 'expenses' AND entity_id IN (SELECT id FROM expenses WHERE vehicle_id = $1))
                OR (module_name = 'damages' AND entity_id IN (SELECT id FROM damages WHERE vehicle_id = $1))
           )",
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

async fn recent_vehicle_tasks(pool: &PgPool, vehicle_id: i64) -> ApiResult<Vec<VehicleTaskRow>> {
    Ok(sqlx::query_as::<_, VehicleTaskRow>(
        "SELECT id, task_type, priority::text AS priority, due_date, task_status, description, created_at
         FROM tasks
         WHERE related_vehicle_id = $1 AND deleted_at IS NULL
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .bind(vehicle_id)
    .fetch_all(pool)
    .await?)
}

async fn recent_vehicle_km_logs(pool: &PgPool, vehicle_id: i64) -> ApiResult<Vec<VehicleKmLogRow>> {
    Ok(sqlx::query_as::<_, VehicleKmLogRow>(
        "SELECT id, km, entry_type::text AS entry_type, verification_status::text AS verification_status, created_at
         FROM km_logs
         WHERE vehicle_id = $1 AND deleted_at IS NULL
         ORDER BY created_at DESC
         LIMIT 10",
    )
    .bind(vehicle_id)
    .fetch_all(pool)
    .await?)
}

async fn sold_vehicle_file_summary(
    pool: &PgPool,
    sold_vehicle_id: i64,
    vehicle_id: i64,
) -> ApiResult<SoldVehicleFileSummary> {
    Ok(sqlx::query_as::<_, SoldVehicleFileSummary>(
        "SELECT
            count(*) AS total_files,
            count(*) FILTER (WHERE module_name = 'vehicles') AS vehicle_files,
            count(*) FILTER (WHERE module_name = 'sold_vehicles' AND file_type = 'sale_document') AS sale_documents,
            count(*) FILTER (WHERE module_name = 'sold_vehicles' AND file_type = 'transfer_document') AS transfer_documents
         FROM file_documents
         WHERE deleted_at IS NULL
           AND (
                (module_name = 'vehicles' AND entity_id = $2)
                OR (module_name = 'sold_vehicles' AND entity_id = $1)
           )",
    )
    .bind(sold_vehicle_id)
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?)
}

fn map_unique_vehicle_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.constraint() == Some("vehicles_plate_key") {
            return ApiError::Conflict("plate already exists".to_string());
        }
    }
    ApiError::Database(err)
}
