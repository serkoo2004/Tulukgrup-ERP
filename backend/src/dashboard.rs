use crate::{auth, error::ApiResult, AppState};
use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_dashboard))
}

#[derive(Debug, Serialize)]
struct DashboardResponse {
    generated_at: DateTime<Utc>,
    vehicles: VehicleDashboard,
    tasks: TaskDashboard,
    policies: PolicyDashboard,
    maintenances: MaintenanceDashboard,
    expenses: ExpenseDashboard,
    damages: DamageDashboard,
    operations: OperationsDashboard,
    inventory: InventoryDashboard,
    support: SupportDashboard,
    warnings: Vec<DashboardWarning>,
    recent_activity: Vec<ActivityRow>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleDashboard {
    total_count: i64,
    active_count: i64,
    passive_count: i64,
    sold_count: i64,
    assigned_count: i64,
    unassigned_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct TaskDashboard {
    open_count: i64,
    critical_count: i64,
    overdue_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct PolicyDashboard {
    active_count: i64,
    ending_in_30_days: i64,
    ended_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct MaintenanceDashboard {
    planned_count: i64,
    scheduled_count: i64,
    completed_count: i64,
    upcoming_by_km_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseDashboard {
    current_month_total: Option<Decimal>,
    pending_count: i64,
    overdue_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct DamageDashboard {
    open_count: i64,
    insurance_count: i64,
    estimated_open_cost: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
struct OperationsDashboard {
    open_inspection_count: i64,
    open_value_loss_count: i64,
    current_month_fuel_total: Option<Decimal>,
    current_month_wash_total: Option<Decimal>,
    pending_quote_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct InventoryDashboard {
    total_products: i64,
    total_stock_value: Option<Decimal>,
    critical_stock_count: i64,
    out_of_stock_count: i64,
    pending_shipments: i64,
    pending_purchase_requests: i64,
    expiring_lots_30_days: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportDashboard {
    open_count: i64,
    in_progress_count: i64,
    waiting_user_count: i64,
    critical_count: i64,
    whatsapp_count: i64,
    response_overdue_count: i64,
    resolution_overdue_count: i64,
    due_soon_count: i64,
    resolved_today_count: i64,
}

#[derive(Debug, Serialize)]
struct DashboardWarning {
    warning_type: String,
    severity: String,
    title: String,
    count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct ActivityRow {
    table_name: String,
    record_id: Option<i64>,
    action_type: String,
    created_by: Option<i64>,
    created_at: DateTime<Utc>,
}

async fn get_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<DashboardResponse>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation", "accounting"])?;

    let vehicles = sqlx::query_as::<_, VehicleDashboard>(
        "SELECT
            count(*) AS total_count,
            count(*) FILTER (WHERE status = 'active'::vehicle_status) AS active_count,
            count(*) FILTER (WHERE status = 'passive'::vehicle_status) AS passive_count,
            count(*) FILTER (WHERE status = 'sold'::vehicle_status) AS sold_count,
            count(*) FILTER (WHERE user_id IS NOT NULL AND status = 'active'::vehicle_status) AS assigned_count,
            count(*) FILTER (WHERE user_id IS NULL AND status = 'active'::vehicle_status) AS unassigned_count
         FROM vehicles
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let tasks = sqlx::query_as::<_, TaskDashboard>(
        "SELECT
            count(*) FILTER (WHERE task_status NOT IN ('completed', 'cancelled')) AS open_count,
            count(*) FILTER (WHERE priority = 'critical'::priority_level AND task_status NOT IN ('completed', 'cancelled')) AS critical_count,
            count(*) FILTER (WHERE due_date < CURRENT_DATE AND task_status NOT IN ('completed', 'cancelled')) AS overdue_count
         FROM tasks
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let policies = sqlx::query_as::<_, PolicyDashboard>(
        "SELECT
            count(*) FILTER (WHERE renewal_status = 'active'::renewal_status) AS active_count,
            count(*) FILTER (WHERE end_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '30 days') AS ending_in_30_days,
            count(*) FILTER (WHERE renewal_status = 'ended'::renewal_status OR end_date < CURRENT_DATE) AS ended_count
         FROM insurance_policies
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let maintenances = sqlx::query_as::<_, MaintenanceDashboard>(
        "SELECT
            count(*) FILTER (WHERE maintenance_status = 'planned'::maintenance_status) AS planned_count,
            count(*) FILTER (WHERE maintenance_status = 'scheduled'::maintenance_status) AS scheduled_count,
            count(*) FILTER (WHERE maintenance_status = 'completed'::maintenance_status) AS completed_count,
            count(*) FILTER (
                WHERE next_maintenance_km IS NOT NULL
                  AND maintenance_status IN ('planned', 'scheduled')
            ) AS upcoming_by_km_count
         FROM maintenances
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let expenses = sqlx::query_as::<_, ExpenseDashboard>(
        "SELECT
            sum(amount) FILTER (
                WHERE expense_date >= date_trunc('month', CURRENT_DATE)::date
                  AND expense_date < (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')::date
            ) AS current_month_total,
            count(*) FILTER (WHERE payment_status = 'pending') AS pending_count,
            count(*) FILTER (WHERE payment_status = 'overdue') AS overdue_count
         FROM expenses
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let damages = sqlx::query_as::<_, DamageDashboard>(
        "SELECT
            count(*) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')) AS open_count,
            count(*) FILTER (WHERE damage_status = 'insurance') AS insurance_count,
            sum(estimated_cost) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')) AS estimated_open_cost
         FROM damages
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let operations = sqlx::query_as::<_, OperationsDashboard>(
        "SELECT
            (SELECT count(*) FROM vehicle_inspections WHERE deleted_at IS NULL AND inspection_status NOT IN ('completed', 'closed', 'cancelled')) AS open_inspection_count,
            (SELECT count(*) FROM value_loss_claims WHERE deleted_at IS NULL AND claim_status NOT IN ('closed', 'cancelled', 'paid')) AS open_value_loss_count,
            (SELECT sum(paid_amount) FROM fuel_entries
             WHERE deleted_at IS NULL
               AND period_year = EXTRACT(YEAR FROM CURRENT_DATE)::int
               AND period_month = EXTRACT(MONTH FROM CURRENT_DATE)::int) AS current_month_fuel_total,
            (SELECT sum(amount) FROM vehicle_washes
             WHERE deleted_at IS NULL
               AND wash_date >= date_trunc('month', CURRENT_DATE)::date
               AND wash_date < (date_trunc('month', CURRENT_DATE) + INTERVAL '1 month')::date) AS current_month_wash_total,
            (SELECT count(*) FROM insurance_quotes WHERE deleted_at IS NULL AND quote_status = 'pending') AS pending_quote_count",
    )
    .fetch_one(&state.pool)
    .await?;

    let inventory = sqlx::query_as::<_, InventoryDashboard>(
        "SELECT
            (SELECT count(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true)::bigint AS total_products,
            (SELECT sum(current_stock * COALESCE(unit_cost, 0)) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true) AS total_stock_value,
            (SELECT count(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true AND current_stock > 0 AND current_stock <= COALESCE(critical_stock, minimum_stock))::bigint AS critical_stock_count,
            (SELECT count(*) FROM inventory_products WHERE deleted_at IS NULL AND is_active = true AND current_stock <= 0)::bigint AS out_of_stock_count,
            (SELECT count(*) FROM inventory_shipments WHERE deleted_at IS NULL AND shipment_status = 'draft')::bigint AS pending_shipments,
            (SELECT count(*) FROM inventory_purchase_requests WHERE deleted_at IS NULL AND request_status = 'open')::bigint AS pending_purchase_requests,
            (SELECT count(*) FROM inventory_lots WHERE deleted_at IS NULL AND expiry_date IS NOT NULL AND expiry_date <= CURRENT_DATE + interval '30 days')::bigint AS expiring_lots_30_days",
    )
    .fetch_one(&state.pool)
    .await?;

    let support = sqlx::query_as::<_, SupportDashboard>(
        "SELECT
            count(*) FILTER (WHERE ticket_status = 'open') AS open_count,
            count(*) FILTER (WHERE ticket_status = 'in_progress') AS in_progress_count,
            count(*) FILTER (WHERE ticket_status = 'waiting_user') AS waiting_user_count,
            count(*) FILTER (WHERE priority = 'critical'::priority_level AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS critical_count,
            count(*) FILTER (WHERE source_channel = 'whatsapp') AS whatsapp_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'response_overdue') AS response_overdue_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'resolution_overdue') AS resolution_overdue_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'due_soon') AS due_soon_count,
            count(*) FILTER (WHERE resolved_at::date = CURRENT_DATE OR closed_at::date = CURRENT_DATE) AS resolved_today_count
         FROM support_tickets
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;

    let recent_activity = sqlx::query_as::<_, ActivityRow>(
        "SELECT table_name, record_id, action_type::text AS action_type, created_by, created_at
         FROM audit_logs
         ORDER BY created_at DESC
         LIMIT 15",
    )
    .fetch_all(&state.pool)
    .await?;

    let warnings = build_warnings(
        &tasks,
        &policies,
        &expenses,
        &damages,
        &operations,
        &inventory,
        &support,
    );

    Ok(Json(DashboardResponse {
        generated_at: Utc::now(),
        vehicles,
        tasks,
        policies,
        maintenances,
        expenses,
        damages,
        operations,
        inventory,
        support,
        warnings,
        recent_activity,
    }))
}

fn build_warnings(
    tasks: &TaskDashboard,
    policies: &PolicyDashboard,
    expenses: &ExpenseDashboard,
    damages: &DamageDashboard,
    operations: &OperationsDashboard,
    inventory: &InventoryDashboard,
    support: &SupportDashboard,
) -> Vec<DashboardWarning> {
    let mut warnings = Vec::new();
    push_warning(
        &mut warnings,
        "tasks_overdue",
        "critical",
        "Geciken görev",
        tasks.overdue_count,
    );
    push_warning(
        &mut warnings,
        "tasks_critical",
        "critical",
        "Kritik açık görev",
        tasks.critical_count,
    );
    push_warning(
        &mut warnings,
        "policies_ending",
        "warning",
        "30 gün içinde biten poliçe",
        policies.ending_in_30_days,
    );
    push_warning(
        &mut warnings,
        "policies_ended",
        "critical",
        "Süresi bitmiş poliçe",
        policies.ended_count,
    );
    push_warning(
        &mut warnings,
        "expenses_overdue",
        "warning",
        "Gecikmiş ödeme",
        expenses.overdue_count,
    );
    push_warning(
        &mut warnings,
        "damages_open",
        "warning",
        "Açık hasar dosyası",
        damages.open_count,
    );
    push_warning(
        &mut warnings,
        "value_loss_open",
        "warning",
        "Açık değer kaybı dosyası",
        operations.open_value_loss_count,
    );
    push_warning(
        &mut warnings,
        "quotes_pending",
        "info",
        "Bekleyen sigorta teklifi",
        operations.pending_quote_count,
    );
    push_warning(
        &mut warnings,
        "inventory_critical",
        "warning",
        "Kritik stok",
        inventory.critical_stock_count,
    );
    push_warning(
        &mut warnings,
        "inventory_empty",
        "critical",
        "Tükenen stok",
        inventory.out_of_stock_count,
    );
    push_warning(
        &mut warnings,
        "inventory_expiring",
        "warning",
        "30 gün içinde SKT lot",
        inventory.expiring_lots_30_days,
    );
    push_warning(
        &mut warnings,
        "inventory_shipments",
        "info",
        "Bekleyen stok sevkiyatı",
        inventory.pending_shipments,
    );
    push_warning(
        &mut warnings,
        "support_critical",
        "critical",
        "Kritik IT destek talebi",
        support.critical_count,
    );
    push_warning(
        &mut warnings,
        "support_sla_resolution",
        "critical",
        "SLA çözüm süresi geçen IT talebi",
        support.resolution_overdue_count,
    );
    push_warning(
        &mut warnings,
        "support_sla_response",
        "warning",
        "SLA ilk cevap süresi geçen IT talebi",
        support.response_overdue_count,
    );
    push_warning(
        &mut warnings,
        "support_sla_due_soon",
        "info",
        "SLA süresi yaklaşan IT talebi",
        support.due_soon_count,
    );
    push_warning(
        &mut warnings,
        "support_open",
        "warning",
        "Açık IT destek talebi",
        support.open_count + support.in_progress_count,
    );
    push_warning(
        &mut warnings,
        "support_waiting",
        "info",
        "Kullanıcı yanıtı bekleyen IT talebi",
        support.waiting_user_count,
    );
    warnings
}

fn push_warning(
    warnings: &mut Vec<DashboardWarning>,
    warning_type: &str,
    severity: &str,
    title: &str,
    count: i64,
) {
    if count > 0 {
        warnings.push(DashboardWarning {
            warning_type: warning_type.to_string(),
            severity: severity.to_string(),
            title: title.to_string(),
            count,
        });
    }
}
