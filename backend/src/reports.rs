use crate::{auth, error::ApiResult, AppState};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/summary", get(get_summary_report))
        .route("/management.xlsx", get(export_management_xlsx))
        .route("/vehicles.xlsx", get(export_vehicles_xlsx))
        .route("/maintenances.xlsx", get(export_maintenances_xlsx))
        .route("/insurance-policies.xlsx", get(export_policies_xlsx))
        .route("/expenses.xlsx", get(export_expenses_xlsx))
        .route("/damages.xlsx", get(export_damages_xlsx))
        .route(
            "/vehicle-inspections.xlsx",
            get(export_vehicle_inspections_xlsx),
        )
        .route(
            "/value-loss-claims.xlsx",
            get(export_value_loss_claims_xlsx),
        )
        .route("/fuel-entries.xlsx", get(export_fuel_entries_xlsx))
        .route("/vehicle-washes.xlsx", get(export_vehicle_washes_xlsx))
        .route("/insurance-quotes.xlsx", get(export_insurance_quotes_xlsx))
        .route(
            "/inventory-products.xlsx",
            get(export_inventory_products_xlsx),
        )
        .route(
            "/inventory-movements.xlsx",
            get(export_inventory_movements_xlsx),
        )
        .route(
            "/inventory-critical.xlsx",
            get(export_inventory_critical_xlsx),
        )
        .route("/support-tickets.xlsx", get(export_support_tickets_xlsx))
        .route(
            "/support-knowledge-base.xlsx",
            get(export_support_knowledge_base_xlsx),
        )
}

#[derive(Debug, serde::Deserialize)]
struct ReportQuery {
    vehicle_id: Option<i64>,
    status: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleExportRow {
    plate: String,
    brand: String,
    model: String,
    model_year: Option<i32>,
    vehicle_type: Option<String>,
    fuel_type: Option<String>,
    transmission: Option<String>,
    company_name: Option<String>,
    department_name: Option<String>,
    assigned_user_name: Option<String>,
    spare_key_location: Option<String>,
    tasitmatik_company: Option<String>,
    has_hgs: bool,
    has_mobiliz: bool,
    has_kopilot: bool,
    has_k2: bool,
    warranty_status: Option<String>,
    warranty_end: Option<NaiveDate>,
    last_km: Option<i32>,
    km_log_count: i64,
    open_task_count: i64,
    ending_policy_count: i64,
    status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct MaintenanceExportRow {
    plate: String,
    maintenance_date: Option<NaiveDate>,
    last_maintenance_km: Option<i32>,
    next_maintenance_km: Option<i32>,
    service_company: Option<String>,
    maintenance_type: Option<String>,
    description: Option<String>,
    total_cost: Option<Decimal>,
    maintenance_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct PolicyExportRow {
    plate: String,
    policy_type: String,
    policy_number: String,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    previous_amount: Option<Decimal>,
    currency: String,
    days_left: Option<i32>,
    renewal_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseExportRow {
    plate: String,
    expense_type: String,
    amount: Decimal,
    invoice_number: Option<String>,
    payment_status: Option<String>,
    expense_date: NaiveDate,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct DamageExportRow {
    plate: String,
    damage_date: NaiveDate,
    damage_type: Option<String>,
    description: Option<String>,
    estimated_cost: Option<Decimal>,
    actual_cost: Option<Decimal>,
    insurance_claim_no: Option<String>,
    damage_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct InspectionExportRow {
    plate: String,
    branch: Option<String>,
    inspection_date: NaiveDate,
    inspector_name: Option<String>,
    exterior_ok: Option<bool>,
    interior_ok: Option<bool>,
    equipment_ok: Option<bool>,
    documents_ok: Option<bool>,
    fee: Option<Decimal>,
    inspection_status: String,
    damage_note: Option<String>,
    action_note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct ValueLossExportRow {
    plate: String,
    accident_date: NaiveDate,
    vehicle_purchase_date: Option<NaiveDate>,
    tramer_amount: Option<Decimal>,
    deprivation_days: Option<i32>,
    requested_amount: Option<Decimal>,
    received_amount: Option<Decimal>,
    claim_status: String,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct FuelEntryExportRow {
    plate: String,
    assigned_user_name: Option<String>,
    period_year: i32,
    period_month: i32,
    fuel_limit: Option<Decimal>,
    paid_amount: Option<Decimal>,
    remaining_limit: Option<Decimal>,
    distance_km: Option<i32>,
    current_km: Option<i32>,
    liter_amount: Option<Decimal>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleWashExportRow {
    plate: String,
    branch: Option<String>,
    wash_company: Option<String>,
    wash_date: NaiveDate,
    user_name: Option<String>,
    amount: Option<Decimal>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct InsuranceQuoteExportRow {
    plate: String,
    quote_type: String,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    gross_premium: Option<Decimal>,
    installment_count: Option<i32>,
    valid_until: Option<NaiveDate>,
    quote_status: String,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct InventoryProductExportRow {
    product_name: String,
    category: Option<String>,
    sub_category: Option<String>,
    brand: Option<String>,
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
}

#[derive(Debug, FromRow, Serialize)]
struct InventoryMovementExportRow {
    product_name: String,
    movement_type: String,
    quantity: Decimal,
    unit: String,
    base_quantity: Decimal,
    previous_stock: Decimal,
    next_stock: Decimal,
    branch_name: Option<String>,
    plate: Option<String>,
    description: Option<String>,
    movement_at: chrono::DateTime<chrono::Utc>,
    cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Serialize)]
struct InventoryCriticalExportRow {
    product_name: String,
    category: Option<String>,
    current_stock: Decimal,
    minimum_stock: Decimal,
    critical_stock: Option<Decimal>,
    maximum_stock: Option<Decimal>,
    alert_level: String,
    suggested_order_quantity: Decimal,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportTicketExportRow {
    ticket_no: String,
    title: String,
    description: String,
    category: String,
    priority: String,
    ticket_status: String,
    source_channel: String,
    reporter_name: Option<String>,
    reporter_phone: Option<String>,
    assigned_user_name: Option<String>,
    assigned_department_name: Option<String>,
    resolution_note: Option<String>,
    sla_status: String,
    sla_response_due_at: Option<chrono::DateTime<chrono::Utc>>,
    sla_resolution_due_at: Option<chrono::DateTime<chrono::Utc>>,
    first_response_at: Option<chrono::DateTime<chrono::Utc>>,
    escalation_level: i32,
    satisfaction_score: Option<i32>,
    satisfaction_note: Option<String>,
    event_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportKnowledgeBaseExportRow {
    title: String,
    category: String,
    problem: String,
    solution: String,
    tags: String,
    source_ticket_no: Option<String>,
    is_published: bool,
    view_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct SummaryReport {
    vehicles: VehicleStatusSummary,
    tasks: TaskSummary,
    policies: PolicySummary,
    maintenances: MaintenanceSummary,
    expenses: ExpenseSummary,
    damages: DamageSummary,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleStatusSummary {
    active_count: i64,
    passive_count: i64,
    sold_count: i64,
    total_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct TaskSummary {
    open_count: i64,
    critical_count: i64,
    overdue_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct PolicySummary {
    active_count: i64,
    ending_in_30_days: i64,
    ended_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct MaintenanceSummary {
    planned_count: i64,
    scheduled_count: i64,
    completed_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseSummary {
    current_month_total: Option<Decimal>,
    pending_count: i64,
    overdue_count: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct DamageSummary {
    open_count: i64,
    insurance_count: i64,
    estimated_open_cost: Option<Decimal>,
}

async fn get_summary_report(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SummaryReport>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    Ok(Json(load_summary_report(&state.pool).await?))
}

async fn load_summary_report(pool: &PgPool) -> ApiResult<SummaryReport> {
    let vehicles = sqlx::query_as::<_, VehicleStatusSummary>(
        "SELECT
            count(*) FILTER (WHERE status = 'active'::vehicle_status) AS active_count,
            count(*) FILTER (WHERE status = 'passive'::vehicle_status) AS passive_count,
            count(*) FILTER (WHERE status = 'sold'::vehicle_status) AS sold_count,
            count(*) AS total_count
         FROM vehicles
         WHERE deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    let tasks = sqlx::query_as::<_, TaskSummary>(
        "SELECT
            count(*) FILTER (WHERE task_status NOT IN ('completed', 'cancelled')) AS open_count,
            count(*) FILTER (WHERE priority = 'critical'::priority_level AND task_status NOT IN ('completed', 'cancelled')) AS critical_count,
            count(*) FILTER (WHERE due_date < CURRENT_DATE AND task_status NOT IN ('completed', 'cancelled')) AS overdue_count
         FROM tasks
         WHERE deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    let policies = sqlx::query_as::<_, PolicySummary>(
        "SELECT
            count(*) FILTER (WHERE renewal_status = 'active'::renewal_status) AS active_count,
            count(*) FILTER (WHERE end_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '30 days') AS ending_in_30_days,
            count(*) FILTER (WHERE renewal_status = 'ended'::renewal_status OR end_date < CURRENT_DATE) AS ended_count
         FROM insurance_policies
         WHERE deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    let maintenances = sqlx::query_as::<_, MaintenanceSummary>(
        "SELECT
            count(*) FILTER (WHERE maintenance_status = 'planned'::maintenance_status) AS planned_count,
            count(*) FILTER (WHERE maintenance_status = 'scheduled'::maintenance_status) AS scheduled_count,
            count(*) FILTER (WHERE maintenance_status = 'completed'::maintenance_status) AS completed_count
         FROM maintenances
         WHERE deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    let expenses = sqlx::query_as::<_, ExpenseSummary>(
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
    .fetch_one(pool)
    .await?;

    let damages = sqlx::query_as::<_, DamageSummary>(
        "SELECT
            count(*) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')) AS open_count,
            count(*) FILTER (WHERE damage_status = 'insurance') AS insurance_count,
            sum(estimated_cost) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')) AS estimated_open_cost
         FROM damages
         WHERE deleted_at IS NULL",
    )
    .fetch_one(pool)
    .await?;

    Ok(SummaryReport {
        vehicles,
        tasks,
        policies,
        maintenances,
        expenses,
        damages,
    })
}

async fn export_management_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let summary = load_summary_report(&state.pool).await?;
    let sheet = vec![
        vec![
            "Alan".to_string(),
            "Ana Sayi / Tutar".to_string(),
            "Takip".to_string(),
            "Kritik".to_string(),
            "Yonetim Notu".to_string(),
        ],
        vec![
            "Araçlar".to_string(),
            format!(
                "{}/{} aktif",
                summary.vehicles.active_count, summary.vehicles.total_count
            ),
            format!("{} pasif", summary.vehicles.passive_count),
            format!("{} satıldı", summary.vehicles.sold_count),
            "Filo canlılığı ve araç durum dağılımı".to_string(),
        ],
        vec![
            "Görevler".to_string(),
            format!("{} açık", summary.tasks.open_count),
            format!("{} gecikmiş", summary.tasks.overdue_count),
            format!("{} kritik", summary.tasks.critical_count),
            "Operasyon takibi ve geciken işler".to_string(),
        ],
        vec![
            "Poliçeler".to_string(),
            format!("{} aktif", summary.policies.active_count),
            format!(
                "{} adet 30 gün içinde bitiyor",
                summary.policies.ending_in_30_days
            ),
            format!("{} bitmiş", summary.policies.ended_count),
            "Trafik, kasko ve yenileme takip özeti".to_string(),
        ],
        vec![
            "Bakımlar".to_string(),
            format!("{} tamamlandı", summary.maintenances.completed_count),
            format!("{} planlı", summary.maintenances.planned_count),
            format!("{} randevulu", summary.maintenances.scheduled_count),
            "Servis, bakım ve planlama özeti".to_string(),
        ],
        vec![
            "Giderler".to_string(),
            format!(
                "{} TRY cari ay",
                optional_to_string(summary.expenses.current_month_total)
            ),
            format!("{} bekleyen", summary.expenses.pending_count),
            format!("{} gecikmiş", summary.expenses.overdue_count),
            "Yakıt, bakım, vergi, ceza ve servis giderleri".to_string(),
        ],
        vec![
            "Hasarlar".to_string(),
            format!("{} açık", summary.damages.open_count),
            format!("{} sigorta sürecinde", summary.damages.insurance_count),
            format!(
                "{} TRY tahmini açık maliyet",
                optional_to_string(summary.damages.estimated_open_cost)
            ),
            "Hasar, ekspertiz ve sigorta dosya takibi".to_string(),
        ],
    ];

    Ok(xlsx_response("management.xlsx", "Management", sheet))
}

async fn export_vehicles_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, VehicleExportRow>(
        "SELECT v.plate, v.brand, v.model, v.model_year, v.vehicle_type, v.fuel_type, v.transmission,
                c.name AS company_name, d.name AS department_name, u.full_name AS assigned_user_name,
                v.spare_key_location, v.tasitmatik_company, v.has_hgs, v.has_mobiliz, v.has_kopilot, v.has_k2,
                v.warranty_status, v.warranty_end,
                (
                    SELECT kl.km
                    FROM km_logs kl
                    WHERE kl.vehicle_id = v.id
                      AND kl.deleted_at IS NULL
                      AND kl.verification_status != 'rejected'::verification_status
                    ORDER BY kl.created_at DESC
                    LIMIT 1
                ) AS last_km,
                (
                    SELECT count(*)
                    FROM km_logs kl
                    WHERE kl.vehicle_id = v.id AND kl.deleted_at IS NULL
                ) AS km_log_count,
                (
                    SELECT count(*)
                    FROM tasks t
                    WHERE t.related_vehicle_id = v.id
                      AND t.deleted_at IS NULL
                      AND t.task_status NOT IN ('completed', 'cancelled')
                ) AS open_task_count,
                (
                    SELECT count(*)
                    FROM insurance_policies p
                    WHERE p.vehicle_id = v.id
                      AND p.deleted_at IS NULL
                      AND p.end_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '30 days'
                ) AS ending_policy_count,
                v.status::text AS status
         FROM vehicles v
         LEFT JOIN companies c ON c.id = v.company_id
         LEFT JOIN departments d ON d.id = v.department_id
         LEFT JOIN users u ON u.id = v.user_id
         WHERE v.deleted_at IS NULL
         ORDER BY v.plate",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Marka".to_string(),
        "Model".to_string(),
        "Model Yili".to_string(),
        "Arac Tipi".to_string(),
        "Yakit Tipi".to_string(),
        "Vites".to_string(),
        "Sirket".to_string(),
        "Departman".to_string(),
        "Kullanan".to_string(),
        "Yedek Anahtar".to_string(),
        "Tasitmatik".to_string(),
        "HGS".to_string(),
        "Mobiliz".to_string(),
        "Kopilot".to_string(),
        "K2".to_string(),
        "Garanti".to_string(),
        "Garanti Bitis".to_string(),
        "Son KM".to_string(),
        "KM Kayit".to_string(),
        "Acik Gorev".to_string(),
        "30 Gun Police".to_string(),
        "Durum".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.brand,
            row.model,
            optional_to_string(row.model_year),
            row.vehicle_type.unwrap_or_default(),
            row.fuel_type.unwrap_or_default(),
            row.transmission.unwrap_or_default(),
            row.company_name.unwrap_or_default(),
            row.department_name.unwrap_or_default(),
            row.assigned_user_name.unwrap_or_default(),
            row.spare_key_location.unwrap_or_default(),
            row.tasitmatik_company.unwrap_or_default(),
            bool_to_string(row.has_hgs),
            bool_to_string(row.has_mobiliz),
            bool_to_string(row.has_kopilot),
            bool_to_string(row.has_k2),
            row.warranty_status.unwrap_or_default(),
            optional_to_string(row.warranty_end),
            optional_to_string(row.last_km),
            row.km_log_count.to_string(),
            row.open_task_count.to_string(),
            row.ending_policy_count.to_string(),
            row.status,
        ]);
    }

    Ok(xlsx_response("vehicles.xlsx", "Vehicles", sheet))
}

async fn export_maintenances_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, MaintenanceExportRow>(
        "SELECT v.plate, m.maintenance_date, m.last_maintenance_km, m.next_maintenance_km,
                m.service_company, m.maintenance_type, m.description,
                m.total_cost, m.maintenance_status::text AS maintenance_status
         FROM maintenances m
         JOIN vehicles v ON v.id = m.vehicle_id
         WHERE m.deleted_at IS NULL
           AND ($1::bigint IS NULL OR m.vehicle_id = $1)
           AND ($2::text IS NULL OR m.maintenance_status::text = $2)
           AND ($3::date IS NULL OR m.maintenance_date >= $3)
           AND ($4::date IS NULL OR m.maintenance_date <= $4)
         ORDER BY m.maintenance_date DESC NULLS LAST, m.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Bakim Tarihi".to_string(),
        "Son Bakim KM".to_string(),
        "Sonraki Bakim KM".to_string(),
        "Servis Firma".to_string(),
        "Bakim Tipi".to_string(),
        "Aciklama".to_string(),
        "Toplam Tutar".to_string(),
        "Durum".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            optional_to_string(row.maintenance_date),
            optional_to_string(row.last_maintenance_km),
            optional_to_string(row.next_maintenance_km),
            row.service_company.unwrap_or_default(),
            row.maintenance_type.unwrap_or_default(),
            row.description.unwrap_or_default(),
            optional_to_string(row.total_cost),
            row.maintenance_status,
        ]);
    }

    Ok(xlsx_response("maintenances.xlsx", "Maintenances", sheet))
}

async fn export_policies_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, PolicyExportRow>(
        "SELECT v.plate, p.policy_type::text AS policy_type, p.policy_number,
                p.insurance_company, p.agency_name, p.start_date, p.end_date,
                p.amount, p.previous_amount, p.currency,
                CASE WHEN p.end_date IS NULL THEN NULL ELSE (p.end_date - CURRENT_DATE) END AS days_left,
                p.renewal_status::text AS renewal_status
         FROM insurance_policies p
         JOIN vehicles v ON v.id = p.vehicle_id
         WHERE p.deleted_at IS NULL
           AND ($1::bigint IS NULL OR p.vehicle_id = $1)
           AND ($2::text IS NULL OR p.renewal_status::text = $2)
           AND ($3::date IS NULL OR p.end_date >= $3)
           AND ($4::date IS NULL OR p.end_date <= $4)
         ORDER BY p.end_date ASC NULLS LAST, p.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Police Tipi".to_string(),
        "Police No".to_string(),
        "Sigorta Sirketi".to_string(),
        "Acente".to_string(),
        "Baslangic".to_string(),
        "Bitis".to_string(),
        "Tutar".to_string(),
        "Onceki Tutar".to_string(),
        "Para Birimi".to_string(),
        "Kalan Gun".to_string(),
        "Yenileme Durumu".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.policy_type,
            row.policy_number,
            row.insurance_company.unwrap_or_default(),
            row.agency_name.unwrap_or_default(),
            optional_to_string(row.start_date),
            optional_to_string(row.end_date),
            optional_to_string(row.amount),
            optional_to_string(row.previous_amount),
            row.currency,
            optional_to_string(row.days_left),
            row.renewal_status,
        ]);
    }

    Ok(xlsx_response("insurance-policies.xlsx", "Insurance", sheet))
}

async fn export_expenses_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, ExpenseExportRow>(
        "SELECT v.plate, e.expense_type::text AS expense_type, e.amount, e.invoice_number,
                e.payment_status, e.expense_date, e.note
         FROM expenses e
         JOIN vehicles v ON v.id = e.vehicle_id
         WHERE e.deleted_at IS NULL
           AND ($1::bigint IS NULL OR e.vehicle_id = $1)
           AND ($2::text IS NULL OR e.payment_status = $2)
           AND ($3::date IS NULL OR e.expense_date >= $3)
           AND ($4::date IS NULL OR e.expense_date <= $4)
         ORDER BY e.expense_date DESC, e.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Gider Tipi".to_string(),
        "Tutar".to_string(),
        "Fatura No".to_string(),
        "Odeme Durumu".to_string(),
        "Gider Tarihi".to_string(),
        "Not".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.expense_type,
            row.amount.to_string(),
            row.invoice_number.unwrap_or_default(),
            row.payment_status.unwrap_or_default(),
            row.expense_date.to_string(),
            row.note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response("expenses.xlsx", "Expenses", sheet))
}

async fn export_damages_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, DamageExportRow>(
        "SELECT v.plate, d.damage_date, d.damage_type, d.description, d.estimated_cost, d.actual_cost,
                d.insurance_claim_no, d.damage_status::text AS damage_status
         FROM damages d
         JOIN vehicles v ON v.id = d.vehicle_id
         WHERE d.deleted_at IS NULL
           AND ($1::bigint IS NULL OR d.vehicle_id = $1)
           AND ($2::text IS NULL OR d.damage_status::text = $2)
           AND ($3::date IS NULL OR d.damage_date >= $3)
           AND ($4::date IS NULL OR d.damage_date <= $4)
         ORDER BY d.damage_date DESC, d.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Hasar Tarihi".to_string(),
        "Hasar Tipi".to_string(),
        "Aciklama".to_string(),
        "Tahmini Tutar".to_string(),
        "Gercek Tutar".to_string(),
        "Sigorta Dosya No".to_string(),
        "Durum".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.damage_date.to_string(),
            row.damage_type.unwrap_or_default(),
            row.description.unwrap_or_default(),
            optional_to_string(row.estimated_cost),
            optional_to_string(row.actual_cost),
            row.insurance_claim_no.unwrap_or_default(),
            row.damage_status,
        ]);
    }

    Ok(xlsx_response("damages.xlsx", "Damages", sheet))
}

async fn export_vehicle_inspections_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, InspectionExportRow>(
        "SELECT v.plate, i.branch, i.inspection_date, u.full_name AS inspector_name,
                i.exterior_ok, i.interior_ok, i.equipment_ok, i.documents_ok,
                i.fee, i.inspection_status, i.damage_note, i.action_note
         FROM vehicle_inspections i
         JOIN vehicles v ON v.id = i.vehicle_id
         LEFT JOIN users u ON u.id = i.inspector_user_id
         WHERE i.deleted_at IS NULL
           AND ($1::bigint IS NULL OR i.vehicle_id = $1)
           AND ($2::text IS NULL OR i.inspection_status = $2)
           AND ($3::date IS NULL OR i.inspection_date >= $3)
           AND ($4::date IS NULL OR i.inspection_date <= $4)
         ORDER BY i.inspection_date DESC, i.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Sube".to_string(),
        "Kontrol Tarihi".to_string(),
        "Kontrol Eden".to_string(),
        "Dis Durum".to_string(),
        "Ic Durum".to_string(),
        "Ekipman".to_string(),
        "Evrak".to_string(),
        "Ucret".to_string(),
        "Durum".to_string(),
        "Hasar Notu".to_string(),
        "Yapilanlar".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.branch.unwrap_or_default(),
            row.inspection_date.to_string(),
            row.inspector_name.unwrap_or_default(),
            optional_bool_to_string(row.exterior_ok),
            optional_bool_to_string(row.interior_ok),
            optional_bool_to_string(row.equipment_ok),
            optional_bool_to_string(row.documents_ok),
            optional_to_string(row.fee),
            row.inspection_status,
            row.damage_note.unwrap_or_default(),
            row.action_note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response(
        "vehicle-inspections.xlsx",
        "Inspections",
        sheet,
    ))
}

async fn export_value_loss_claims_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, ValueLossExportRow>(
        "SELECT v.plate, c.accident_date, c.vehicle_purchase_date, c.tramer_amount,
                c.deprivation_days, c.requested_amount, c.received_amount, c.claim_status, c.note
         FROM value_loss_claims c
         JOIN vehicles v ON v.id = c.vehicle_id
         WHERE c.deleted_at IS NULL
           AND ($1::bigint IS NULL OR c.vehicle_id = $1)
           AND ($2::text IS NULL OR c.claim_status = $2)
           AND ($3::date IS NULL OR c.accident_date >= $3)
           AND ($4::date IS NULL OR c.accident_date <= $4)
         ORDER BY c.accident_date DESC, c.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Kaza Tarihi".to_string(),
        "Arac Alim Tarihi".to_string(),
        "Tramer Kaydi".to_string(),
        "Hak Mahrumiyeti Gun".to_string(),
        "Talep Tutar".to_string(),
        "Gelen Tutar".to_string(),
        "Durum".to_string(),
        "Not".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.accident_date.to_string(),
            optional_to_string(row.vehicle_purchase_date),
            optional_to_string(row.tramer_amount),
            optional_to_string(row.deprivation_days),
            optional_to_string(row.requested_amount),
            optional_to_string(row.received_amount),
            row.claim_status,
            row.note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response("value-loss-claims.xlsx", "ValueLoss", sheet))
}

async fn export_fuel_entries_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, FuelEntryExportRow>(
        "SELECT v.plate, u.full_name AS assigned_user_name, f.period_year, f.period_month,
                f.fuel_limit, f.paid_amount, f.remaining_limit, f.distance_km,
                f.current_km, f.liter_amount, f.note
         FROM fuel_entries f
         JOIN vehicles v ON v.id = f.vehicle_id
         LEFT JOIN users u ON u.id = v.user_id
         WHERE f.deleted_at IS NULL
           AND ($1::bigint IS NULL OR f.vehicle_id = $1)
           AND ($2::date IS NULL OR make_date(f.period_year, f.period_month, 1) >= date_trunc('month', $2::date)::date)
           AND ($3::date IS NULL OR make_date(f.period_year, f.period_month, 1) <= date_trunc('month', $3::date)::date)
         ORDER BY f.period_year DESC, f.period_month DESC, v.plate",
    )
    .bind(query.vehicle_id)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Kullanici".to_string(),
        "Yil".to_string(),
        "Ay".to_string(),
        "Yakit Limiti".to_string(),
        "Odenen Yakit".to_string(),
        "Kalan Limit".to_string(),
        "Gidilen KM".to_string(),
        "Guncel KM".to_string(),
        "Litre".to_string(),
        "Not".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.assigned_user_name.unwrap_or_default(),
            row.period_year.to_string(),
            row.period_month.to_string(),
            optional_to_string(row.fuel_limit),
            optional_to_string(row.paid_amount),
            optional_to_string(row.remaining_limit),
            optional_to_string(row.distance_km),
            optional_to_string(row.current_km),
            optional_to_string(row.liter_amount),
            row.note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response("fuel-entries.xlsx", "Fuel", sheet))
}

async fn export_vehicle_washes_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, VehicleWashExportRow>(
        "SELECT v.plate, w.branch, w.wash_company, w.wash_date, u.full_name AS user_name, w.amount, w.note
         FROM vehicle_washes w
         JOIN vehicles v ON v.id = w.vehicle_id
         LEFT JOIN users u ON u.id = w.user_id
         WHERE w.deleted_at IS NULL
           AND ($1::bigint IS NULL OR w.vehicle_id = $1)
           AND ($2::date IS NULL OR w.wash_date >= $2)
           AND ($3::date IS NULL OR w.wash_date <= $3)
         ORDER BY w.wash_date DESC, w.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Sube".to_string(),
        "Yikama Firmasi".to_string(),
        "Yikama Tarihi".to_string(),
        "Kullanici".to_string(),
        "Ucret".to_string(),
        "Not".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.branch.unwrap_or_default(),
            row.wash_company.unwrap_or_default(),
            row.wash_date.to_string(),
            row.user_name.unwrap_or_default(),
            optional_to_string(row.amount),
            row.note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response("vehicle-washes.xlsx", "Washes", sheet))
}

async fn export_insurance_quotes_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, InsuranceQuoteExportRow>(
        "SELECT v.plate, q.quote_type, q.insurance_company, q.agency_name, q.gross_premium,
                q.installment_count, q.valid_until, q.quote_status, q.note
         FROM insurance_quotes q
         JOIN vehicles v ON v.id = q.vehicle_id
         WHERE q.deleted_at IS NULL
           AND ($1::bigint IS NULL OR q.vehicle_id = $1)
           AND ($2::text IS NULL OR q.quote_status = $2)
           AND ($3::date IS NULL OR q.valid_until >= $3)
           AND ($4::date IS NULL OR q.valid_until <= $4)
         ORDER BY q.valid_until ASC NULLS LAST, q.created_at DESC",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Plaka".to_string(),
        "Teklif Tipi".to_string(),
        "Sigorta Sirketi".to_string(),
        "Acente".to_string(),
        "Brut Prim".to_string(),
        "Taksit".to_string(),
        "Gecerlilik".to_string(),
        "Durum".to_string(),
        "Not".to_string(),
    ]];

    for row in rows {
        sheet.push(vec![
            row.plate,
            row.quote_type,
            row.insurance_company.unwrap_or_default(),
            row.agency_name.unwrap_or_default(),
            optional_to_string(row.gross_premium),
            optional_to_string(row.installment_count),
            optional_to_string(row.valid_until),
            row.quote_status,
            row.note.unwrap_or_default(),
        ]);
    }

    Ok(xlsx_response("insurance-quotes.xlsx", "Quotes", sheet))
}

async fn export_inventory_products_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    authorize_report(&state, &headers).await?;
    let rows = sqlx::query_as::<_, InventoryProductExportRow>(
        "SELECT product_name, category, sub_category, brand, main_unit,
                package_unit, package_multiplier, current_stock, available_stock, reserved_stock,
                minimum_stock, maximum_stock, critical_stock, safety_stock, unit_cost,
                is_lot_tracked, expiry_tracking, is_active
         FROM inventory_products
         WHERE deleted_at IS NULL
         ORDER BY category NULLS LAST, product_name",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Ürün Adı".to_string(),
        "Kategori".to_string(),
        "Alt Kategori".to_string(),
        "Marka".to_string(),
        "Ana Birim".to_string(),
        "Koli/Paket Birimi".to_string(),
        "Çarpan".to_string(),
        "Mevcut Stok".to_string(),
        "Kullanılabilir".to_string(),
        "Rezerve".to_string(),
        "Minimum".to_string(),
        "Maksimum".to_string(),
        "Kritik".to_string(),
        "Güvenlik".to_string(),
        "Birim Maliyet".to_string(),
        "Lot Takip".to_string(),
        "SKT Takip".to_string(),
        "Durum".to_string(),
    ]];
    for row in rows {
        sheet.push(vec![
            row.product_name,
            row.category.unwrap_or_default(),
            row.sub_category.unwrap_or_default(),
            row.brand.unwrap_or_default(),
            row.main_unit,
            row.package_unit.unwrap_or_default(),
            optional_decimal_to_string(row.package_multiplier),
            decimal_to_string(row.current_stock),
            decimal_to_string(row.available_stock),
            decimal_to_string(row.reserved_stock),
            decimal_to_string(row.minimum_stock),
            optional_decimal_to_string(row.maximum_stock),
            optional_decimal_to_string(row.critical_stock),
            optional_decimal_to_string(row.safety_stock),
            optional_to_string(row.unit_cost),
            bool_to_string(row.is_lot_tracked),
            bool_to_string(row.expiry_tracking),
            if row.is_active { "Aktif" } else { "Pasif" }.to_string(),
        ]);
    }
    Ok(xlsx_response(
        "inventory-products.xlsx",
        "InventoryProducts",
        sheet,
    ))
}

async fn export_inventory_movements_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    authorize_report(&state, &headers).await?;
    let rows = sqlx::query_as::<_, InventoryMovementExportRow>(
        "SELECT p.product_name, m.movement_type, m.quantity, m.unit, m.base_quantity,
                m.previous_stock, m.next_stock, d.name AS branch_name, v.plate, m.description,
                m.movement_at, m.cancelled_at
         FROM inventory_movements m
         JOIN inventory_products p ON p.id = m.product_id
         LEFT JOIN departments d ON d.id = m.branch_id
         LEFT JOIN vehicles v ON v.id = m.vehicle_id
         WHERE ($1::date IS NULL OR m.movement_at::date >= $1)
           AND ($2::date IS NULL OR m.movement_at::date <= $2)
         ORDER BY m.movement_at DESC
         LIMIT 1000",
    )
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Ürün".to_string(),
        "Hareket".to_string(),
        "Miktar".to_string(),
        "Birim".to_string(),
        "Ana Miktar".to_string(),
        "Önceki Stok".to_string(),
        "Sonraki Stok".to_string(),
        "Şube".to_string(),
        "Araç".to_string(),
        "Açıklama".to_string(),
        "Tarih".to_string(),
        "İptal".to_string(),
    ]];
    for row in rows {
        sheet.push(vec![
            row.product_name,
            row.movement_type,
            decimal_to_string(row.quantity),
            row.unit,
            decimal_to_string(row.base_quantity),
            decimal_to_string(row.previous_stock),
            decimal_to_string(row.next_stock),
            row.branch_name.unwrap_or_default(),
            row.plate.unwrap_or_default(),
            row.description.unwrap_or_default(),
            row.movement_at.to_string(),
            optional_to_string(row.cancelled_at),
        ]);
    }
    Ok(xlsx_response(
        "inventory-movements.xlsx",
        "InventoryMoves",
        sheet,
    ))
}

async fn export_inventory_critical_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    authorize_report(&state, &headers).await?;
    let rows = sqlx::query_as::<_, InventoryCriticalExportRow>(
        "SELECT product_name, category, current_stock, minimum_stock, critical_stock, maximum_stock,
                CASE WHEN current_stock <= 0 THEN 'Kırmızı'
                     WHEN current_stock < minimum_stock THEN 'Turuncu'
                     ELSE 'Sarı' END AS alert_level,
                GREATEST(COALESCE(maximum_stock, minimum_stock * 2) - current_stock, 0) AS suggested_order_quantity
         FROM inventory_products
         WHERE deleted_at IS NULL AND is_active = true
           AND current_stock <= GREATEST(minimum_stock, COALESCE(critical_stock, minimum_stock))
         ORDER BY current_stock ASC, product_name",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Ürün".to_string(),
        "Kategori".to_string(),
        "Mevcut Stok".to_string(),
        "Minimum".to_string(),
        "Kritik".to_string(),
        "Maksimum".to_string(),
        "Alarm".to_string(),
        "Önerilen Sipariş".to_string(),
    ]];
    for row in rows {
        sheet.push(vec![
            row.product_name,
            row.category.unwrap_or_default(),
            decimal_to_string(row.current_stock),
            decimal_to_string(row.minimum_stock),
            optional_decimal_to_string(row.critical_stock),
            optional_decimal_to_string(row.maximum_stock),
            row.alert_level,
            decimal_to_string(row.suggested_order_quantity),
        ]);
    }
    Ok(xlsx_response(
        "inventory-critical.xlsx",
        "InventoryCritical",
        sheet,
    ))
}

async fn export_support_tickets_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ReportQuery>,
) -> ApiResult<Response> {
    authorize_report(&state, &headers).await?;
    let rows = sqlx::query_as::<_, SupportTicketExportRow>(
        "SELECT t.ticket_no, t.title, t.description, t.category, t.priority::text AS priority,
                t.ticket_status, t.source_channel, t.reporter_name, t.reporter_phone,
                u.full_name AS assigned_user_name, d.name AS assigned_department_name,
                t.resolution_note,
                support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at) AS sla_status,
                t.sla_response_due_at, t.sla_resolution_due_at, t.first_response_at,
                t.escalation_level, t.satisfaction_score, t.satisfaction_note,
                (SELECT count(*) FROM support_ticket_events e WHERE e.ticket_id = t.id) AS event_count,
                t.created_at, t.resolved_at, t.closed_at
         FROM support_tickets t
         LEFT JOIN users u ON u.id = t.assigned_user_id
         LEFT JOIN departments d ON d.id = t.assigned_department_id
         WHERE t.deleted_at IS NULL
           AND ($1::text IS NULL OR t.ticket_status = $1)
           AND ($2::date IS NULL OR t.created_at::date >= $2)
           AND ($3::date IS NULL OR t.created_at::date <= $3)
         ORDER BY
           CASE t.ticket_status
             WHEN 'open' THEN 1
             WHEN 'in_progress' THEN 2
             WHEN 'waiting_user' THEN 3
             WHEN 'resolved' THEN 4
             ELSE 5
           END,
           CASE t.priority::text
             WHEN 'critical' THEN 1
             WHEN 'high' THEN 2
             WHEN 'medium' THEN 3
             ELSE 4
           END,
           t.created_at DESC
         LIMIT 2000",
    )
    .bind(query.status)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Talep No".to_string(),
        "Başlık".to_string(),
        "Açıklama".to_string(),
        "Kategori".to_string(),
        "Öncelik".to_string(),
        "Durum".to_string(),
        "Kaynak".to_string(),
        "Bildiren".to_string(),
        "Telefon".to_string(),
        "Atanan".to_string(),
        "Birim".to_string(),
        "SLA Durumu".to_string(),
        "İlk Cevap Hedefi".to_string(),
        "Çözüm Hedefi".to_string(),
        "İlk Cevap".to_string(),
        "Eskalasyon".to_string(),
        "Çözüm Notu".to_string(),
        "Memnuniyet".to_string(),
        "Memnuniyet Notu".to_string(),
        "Olay Sayısı".to_string(),
        "Açılış".to_string(),
        "Çözüm".to_string(),
        "Kapanış".to_string(),
    ]];
    for row in rows {
        sheet.push(vec![
            row.ticket_no,
            row.title,
            row.description,
            row.category,
            row.priority,
            row.ticket_status,
            row.source_channel,
            row.reporter_name.unwrap_or_default(),
            row.reporter_phone.unwrap_or_default(),
            row.assigned_user_name.unwrap_or_default(),
            row.assigned_department_name.unwrap_or_default(),
            row.sla_status,
            optional_to_string(row.sla_response_due_at),
            optional_to_string(row.sla_resolution_due_at),
            optional_to_string(row.first_response_at),
            row.escalation_level.to_string(),
            row.resolution_note.unwrap_or_default(),
            optional_to_string(row.satisfaction_score),
            row.satisfaction_note.unwrap_or_default(),
            row.event_count.to_string(),
            row.created_at.to_string(),
            optional_to_string(row.resolved_at),
            optional_to_string(row.closed_at),
        ]);
    }
    Ok(xlsx_response(
        "support-tickets.xlsx",
        "SupportTickets",
        sheet,
    ))
}

async fn export_support_knowledge_base_xlsx(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    authorize_report(&state, &headers).await?;
    let rows = sqlx::query_as::<_, SupportKnowledgeBaseExportRow>(
        "SELECT kb.title, kb.category, kb.problem, kb.solution,
                COALESCE(
                  (
                    SELECT string_agg(value, ', ')
                    FROM jsonb_array_elements_text(kb.tags) AS value
                  ),
                  ''
                ) AS tags,
                t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at
         FROM support_knowledge_base kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id
         WHERE kb.deleted_at IS NULL
         ORDER BY kb.is_published DESC, kb.updated_at DESC
         LIMIT 2000",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut sheet = vec![vec![
        "Başlık".to_string(),
        "Kategori".to_string(),
        "Problem".to_string(),
        "Çözüm".to_string(),
        "Etiketler".to_string(),
        "Kaynak Talep".to_string(),
        "Yayın".to_string(),
        "Okunma".to_string(),
        "Oluşturma".to_string(),
        "Güncelleme".to_string(),
    ]];
    for row in rows {
        sheet.push(vec![
            row.title,
            row.category,
            row.problem,
            row.solution,
            row.tags,
            row.source_ticket_no.unwrap_or_default(),
            bool_to_string(row.is_published),
            row.view_count.to_string(),
            row.created_at.to_string(),
            row.updated_at.to_string(),
        ]);
    }
    Ok(xlsx_response(
        "support-knowledge-base.xlsx",
        "SupportKnowledge",
        sheet,
    ))
}

fn optional_to_string<T: ToString>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn decimal_to_string(value: Decimal) -> String {
    value.normalize().to_string()
}

fn optional_decimal_to_string(value: Option<Decimal>) -> String {
    value.map(decimal_to_string).unwrap_or_default()
}

async fn authorize_report(state: &AppState, headers: &HeaderMap) -> ApiResult<()> {
    let current = auth::current_user(state, headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(())
}

fn bool_to_string(value: bool) -> String {
    if value { "Evet" } else { "Hayır" }.to_string()
}

fn optional_bool_to_string(value: Option<bool>) -> String {
    value.map(bool_to_string).unwrap_or_default()
}

fn xlsx_response(filename: &str, sheet_name: &str, rows: Vec<Vec<String>>) -> Response {
    let bytes = build_xlsx(report_title(sheet_name), sheet_name, rows);
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    (StatusCode::OK, headers, bytes).into_response()
}

fn report_title(sheet_name: &str) -> &'static str {
    match sheet_name {
        "Vehicles" => "TULUKLAR GROUP - ARAÇ RAPORU",
        "Maintenances" => "TULUKLAR GROUP - BAKIM RAPORU",
        "Insurance" => "TULUKLAR GROUP - POLİÇE RAPORU",
        "Expenses" => "TULUKLAR GROUP - GİDER RAPORU",
        "Damages" => "TULUKLAR GROUP - HASAR RAPORU",
        "Management" => "TULUKLAR GROUP - YÖNETİM ÖZET RAPORU",
        "Inspections" => "TULUKLAR GROUP - ARAÇ KONTROL RAPORU",
        "ValueLoss" => "TULUKLAR GROUP - DEĞER KAYBI RAPORU",
        "Fuel" => "TULUKLAR GROUP - YAKIT KM RAPORU",
        "Washes" => "TULUKLAR GROUP - ARAÇ YIKAMA RAPORU",
        "Quotes" => "TULUKLAR GROUP - SİGORTA TEKLİF RAPORU",
        "InventoryProducts" => "TULUKLAR GROUP - STOK ÜRÜN RAPORU",
        "InventoryMoves" => "TULUKLAR GROUP - STOK HAREKET RAPORU",
        "InventoryCritical" => "TULUKLAR GROUP - KRİTİK STOK RAPORU",
        "SupportTickets" => "TULUKLAR GROUP - IT DESTEK TALEP RAPORU",
        "SupportKnowledge" => "TULUKLAR GROUP - IT DESTEK BİLGİ BANKASI RAPORU",
        _ => "TULUKLAR GROUP - ERP RAPORU",
    }
}

fn build_xlsx(title: &str, sheet_name: &str, rows: Vec<Vec<String>>) -> Vec<u8> {
    let sheet_name = escape_xml(sheet_name);
    let sheet_xml = worksheet_xml(title, &rows);
    let workbook_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="{}" sheetId="1" r:id="rId1"/>
  </sheets>
</workbook>"#,
        sheet_name
    );

    let files = vec![
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#
                .as_bytes()
                .to_vec(),
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#
                .as_bytes()
                .to_vec(),
        ),
        ("xl/workbook.xml", workbook_xml.into_bytes()),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#
                .as_bytes()
                .to_vec(),
        ),
        (
            "xl/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="6">
    <font><sz val="11"/><name val="Calibri"/></font>
    <font><b/><sz val="11"/><name val="Calibri"/><color rgb="FFFFFFFF"/></font>
    <font><b/><sz val="16"/><name val="Calibri"/><color rgb="FFFFFFFF"/></font>
    <font><b/><sz val="11"/><name val="Calibri"/><color rgb="FF1F2937"/></font>
    <font><b/><sz val="14"/><name val="Calibri"/><color rgb="FF111827"/></font>
    <font><sz val="10"/><name val="Calibri"/><color rgb="FF4B5563"/></font>
  </fonts>
  <fills count="10">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="gray125"/></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FF1F4E78"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFD9EAF7"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFEFF6FB"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFF7FBFD"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFE2F0D9"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFFFF2CC"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFF4CCCC"/><bgColor indexed="64"/></patternFill></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FFE7E6E6"/><bgColor indexed="64"/></patternFill></fill>
  </fills>
  <borders count="2">
    <border><left/><right/><top/><bottom/><diagonal/></border>
    <border><left style="thin"><color rgb="FFD9E2EC"/></left><right style="thin"><color rgb="FFD9E2EC"/></right><top style="thin"><color rgb="FFD9E2EC"/></top><bottom style="thin"><color rgb="FFD9E2EC"/></bottom><diagonal/></border>
  </borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="12">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    <xf numFmtId="0" fontId="2" fillId="2" borderId="0" xfId="0" applyFont="1" applyFill="1"><alignment horizontal="center" vertical="center"/></xf>
    <xf numFmtId="0" fontId="5" fillId="0" borderId="0" xfId="0" applyFont="1"><alignment horizontal="right"/></xf>
    <xf numFmtId="0" fontId="3" fillId="3" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
    <xf numFmtId="0" fontId="4" fillId="3" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
    <xf numFmtId="0" fontId="1" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center" vertical="center"/></xf>
    <xf numFmtId="0" fontId="0" fillId="5" borderId="1" xfId="0" applyFill="1" applyBorder="1"><alignment vertical="center"/></xf>
    <xf numFmtId="0" fontId="0" fillId="4" borderId="1" xfId="0" applyFill="1" applyBorder="1"><alignment vertical="center"/></xf>
    <xf numFmtId="0" fontId="3" fillId="6" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
    <xf numFmtId="0" fontId="3" fillId="7" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
    <xf numFmtId="0" fontId="3" fillId="8" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
    <xf numFmtId="0" fontId="3" fillId="9" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1"><alignment horizontal="center"/></xf>
  </cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
  <dxfs count="0"/>
  <tableStyles count="0" defaultTableStyle="TableStyleMedium2" defaultPivotStyle="PivotStyleLight16"/>
</styleSheet>"#
                .as_bytes()
                .to_vec(),
        ),
        ("xl/worksheets/sheet1.xml", sheet_xml.into_bytes()),
    ];

    zip_store(files)
}

fn worksheet_xml(title: &str, rows: &[Vec<String>]) -> String {
    let headers = rows.first().cloned().unwrap_or_default();
    let data_rows = if rows.len() > 1 { &rows[1..] } else { &[] };
    let col_count = headers.len().max(1);
    let table_header_row = 7usize;
    let first_data_row = table_header_row + 1;
    let last_row = table_header_row + data_rows.len();
    let last_col = column_name(col_count - 1);
    let metrics = report_metrics(&headers, data_rows);

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetPr><pageSetUpPr fitToPage="1"/></sheetPr>
  <sheetViews><sheetView workbookViewId="0"><pane ySplit="7" topLeftCell="A8" activePane="bottomLeft" state="frozen"/></sheetView></sheetViews>
  <sheetFormatPr defaultRowHeight="15"/>"#,
    );

    xml.push_str("<cols>");
    for col_index in 0..col_count {
        let width = column_width(headers.get(col_index).map(String::as_str).unwrap_or(""));
        xml.push_str(&format!(
            "<col min=\"{}\" max=\"{}\" width=\"{}\" customWidth=\"1\"/>",
            col_index + 1,
            col_index + 1,
            width
        ));
    }
    xml.push_str("</cols><sheetData>");

    xml.push_str("<row r=\"1\" ht=\"28\" customHeight=\"1\">");
    xml.push_str(&cell_xml(0, 1, title, 1));
    xml.push_str("</row>");

    xml.push_str("<row r=\"2\">");
    xml.push_str(&cell_xml(0, 2, "Oluşturulma zamanı", 2));
    xml.push_str(&cell_xml(
        1,
        2,
        &chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        2,
    ));
    xml.push_str("</row>");

    xml.push_str("<row r=\"4\">");
    for (idx, label) in metrics.iter().map(|metric| metric.0.as_str()).enumerate() {
        xml.push_str(&cell_xml(idx, 4, label, 3));
    }
    xml.push_str("</row>");

    xml.push_str("<row r=\"5\" ht=\"24\" customHeight=\"1\">");
    for (idx, value) in metrics.iter().map(|metric| metric.1.as_str()).enumerate() {
        xml.push_str(&cell_xml(idx, 5, value, 4));
    }
    xml.push_str("</row>");

    xml.push_str(&format!(
        "<row r=\"{}\" ht=\"22\" customHeight=\"1\">",
        table_header_row
    ));
    for (col_index, value) in headers.iter().enumerate() {
        xml.push_str(&cell_xml(col_index, table_header_row, value, 5));
    }
    xml.push_str("</row>");

    if data_rows.is_empty() {
        xml.push_str(&format!("<row r=\"{}\">", first_data_row));
        xml.push_str(&cell_xml(
            0,
            first_data_row,
            "Bu rapor icin kayit bulunamadi.",
            11,
        ));
        xml.push_str("</row>");
    }

    for (row_index, row) in data_rows.iter().enumerate() {
        let excel_row = first_data_row + row_index;
        xml.push_str(&format!("<row r=\"{}\">", excel_row));
        for (col_index, value) in row.iter().enumerate() {
            let style = data_cell_style(row_index, col_index, &headers, value);
            xml.push_str(&cell_xml(col_index, excel_row, value, style));
        }
        xml.push_str("</row>");
    }

    xml.push_str("</sheetData>");
    if col_count > 1 {
        xml.push_str(&format!(
            "<mergeCells count=\"1\"><mergeCell ref=\"A1:{}1\"/></mergeCells>",
            last_col
        ));
    }
    let filter_last_row = if data_rows.is_empty() {
        first_data_row
    } else {
        last_row
    };
    xml.push_str(&format!(
        "<autoFilter ref=\"A{}:{}{}\"/>",
        table_header_row, last_col, filter_last_row
    ));
    xml.push_str("<printOptions horizontalCentered=\"1\"/><pageMargins left=\"0.35\" right=\"0.35\" top=\"0.55\" bottom=\"0.55\" header=\"0.3\" footer=\"0.3\"/><pageSetup orientation=\"landscape\" fitToWidth=\"1\" fitToHeight=\"0\"/></worksheet>");
    xml
}

fn cell_xml(col_index: usize, row_index: usize, value: &str, style_id: usize) -> String {
    format!(
        "<c r=\"{}{}\" t=\"inlineStr\" s=\"{}\"><is><t>{}</t></is></c>",
        column_name(col_index),
        row_index,
        style_id,
        escape_xml(value)
    )
}

fn column_width(header: &str) -> usize {
    let base = match header {
        "Plaka" => 14,
        "Marka" | "Model" | "Durum" => 18,
        "Not" | "Aciklama" | "Yonetim Notu" => 42,
        "Sigorta Sirketi" | "Servis Firma" | "Sigorta Dosya No" | "Kullanan" => 24,
        "Sirket" | "Departman" => 22,
        "Yenileme Durumu" | "Odeme Durumu" => 20,
        "Ana Sayi / Tutar" => 24,
        _ => 16,
    };
    base.max(header.chars().count() + 3).min(45)
}

fn report_metrics(headers: &[String], rows: &[Vec<String>]) -> Vec<(String, String)> {
    let status_index = headers
        .iter()
        .position(|header| header.contains("Durum") || header.contains("Status"));
    let mut positive = 0usize;
    let mut warning = 0usize;
    let mut closed = 0usize;

    if let Some(index) = status_index {
        for row in rows {
            let status = row
                .get(index)
                .map(|value| value.to_lowercase())
                .unwrap_or_default();
            if status.contains("active")
                || status.contains("open")
                || status.contains("planned")
                || status.contains("scheduled")
                || status.contains("verified")
            {
                positive += 1;
            } else if status.contains("pending")
                || status.contains("approaching")
                || status.contains("insurance")
                || status.contains("expertise")
                || status.contains("overdue")
            {
                warning += 1;
            } else if !status.trim().is_empty() {
                closed += 1;
            }
        }
    }

    vec![
        ("Toplam Kayıt".to_string(), rows.len().to_string()),
        ("Aktif / Açık".to_string(), positive.to_string()),
        ("Uyarı / Takip".to_string(), warning.to_string()),
        ("Kapalı / Diğer".to_string(), closed.to_string()),
    ]
}

fn data_cell_style(row_index: usize, col_index: usize, headers: &[String], value: &str) -> usize {
    let lower = value.to_lowercase();
    let is_status_col = headers
        .get(col_index)
        .map(|header| header.contains("Durum") || header.contains("Status"))
        .unwrap_or(false);

    if is_status_col {
        if lower.contains("active")
            || lower.contains("verified")
            || lower.contains("completed")
            || lower.contains("closed")
        {
            return 8;
        }
        if lower.contains("pending")
            || lower.contains("approaching")
            || lower.contains("planned")
            || lower.contains("scheduled")
            || lower.contains("insurance")
            || lower.contains("expertise")
        {
            return 9;
        }
        if lower.contains("rejected")
            || lower.contains("cancelled")
            || lower.contains("overdue")
            || lower.contains("ended")
            || lower.contains("suspicious")
        {
            return 10;
        }
        if lower.contains("sold") || lower.contains("passive") {
            return 11;
        }
    }

    if row_index % 2 == 0 {
        6
    } else {
        7
    }
}

fn column_name(mut index: usize) -> String {
    let mut name = String::new();
    loop {
        let remainder = index % 26;
        name.insert(0, (b'A' + remainder as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    name
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn zip_store(files: Vec<(&'static str, Vec<u8>)>) -> Vec<u8> {
    let mut output = Vec::new();
    let mut central_directory = Vec::new();
    let file_count = files.len() as u16;

    for (name, data) in files {
        let offset = output.len() as u32;
        let name_bytes = name.as_bytes();
        let crc = crc32(&data);
        let size = data.len() as u32;

        write_u32(&mut output, 0x04034b50);
        write_u16(&mut output, 20);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u16(&mut output, 0);
        write_u32(&mut output, crc);
        write_u32(&mut output, size);
        write_u32(&mut output, size);
        write_u16(&mut output, name_bytes.len() as u16);
        write_u16(&mut output, 0);
        output.extend_from_slice(name_bytes);
        output.extend_from_slice(&data);

        write_u32(&mut central_directory, 0x02014b50);
        write_u16(&mut central_directory, 20);
        write_u16(&mut central_directory, 20);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u32(&mut central_directory, crc);
        write_u32(&mut central_directory, size);
        write_u32(&mut central_directory, size);
        write_u16(&mut central_directory, name_bytes.len() as u16);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u16(&mut central_directory, 0);
        write_u32(&mut central_directory, 0);
        write_u32(&mut central_directory, offset);
        central_directory.extend_from_slice(name_bytes);
    }

    let central_offset = output.len() as u32;
    let central_size = central_directory.len() as u32;
    output.extend_from_slice(&central_directory);
    write_u32(&mut output, 0x06054b50);
    write_u16(&mut output, 0);
    write_u16(&mut output, 0);
    write_u16(&mut output, file_count);
    write_u16(&mut output, file_count);
    write_u32(&mut output, central_size);
    write_u32(&mut output, central_offset);
    write_u16(&mut output, 0);
    output
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}
