use crate::{auth, error::ApiResult, AppState};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(global_search))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct SearchResult {
    result_type: String,
    id: i64,
    title: String,
    subtitle: Option<String>,
    status: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

async fn global_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<SearchResult>>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation", "accounting"])?;

    let q = format!("%{}%", query.q.trim());
    let limit = query.limit.unwrap_or(30).clamp(1, 100);
    let rows = sqlx::query_as::<_, SearchResult>(
        "SELECT * FROM (
            SELECT 'vehicle'::text AS result_type, id, plate AS title,
                   concat_ws(' ', brand, model, model_year::text) AS subtitle,
                   status::text AS status, created_at
            FROM vehicles
            WHERE deleted_at IS NULL
              AND (plate ILIKE $1 OR brand ILIKE $1 OR model ILIKE $1 OR chassis_no ILIKE $1 OR engine_no ILIKE $1)

            UNION ALL

            SELECT 'user'::text AS result_type, id, full_name AS title,
                   email AS subtitle, role::text AS status, created_at
            FROM users
            WHERE deleted_at IS NULL
              AND (full_name ILIKE $1 OR email ILIKE $1 OR phone ILIKE $1)

            UNION ALL

            SELECT 'policy'::text AS result_type, p.id, p.policy_number AS title,
                   concat(v.plate, ' ', COALESCE(p.insurance_company, '')) AS subtitle,
                   p.renewal_status::text AS status, p.created_at
            FROM insurance_policies p
            JOIN vehicles v ON v.id = p.vehicle_id
            WHERE p.deleted_at IS NULL
              AND (p.policy_number ILIKE $1 OR p.insurance_company ILIKE $1 OR p.agency_name ILIKE $1 OR v.plate ILIKE $1)

            UNION ALL

            SELECT 'damage'::text AS result_type, d.id, v.plate AS title,
                   concat_ws(' ', d.damage_type, d.insurance_claim_no, d.description) AS subtitle,
                   d.damage_status::text AS status, d.created_at
            FROM damages d
            JOIN vehicles v ON v.id = d.vehicle_id
            WHERE d.deleted_at IS NULL
              AND (v.plate ILIKE $1 OR d.insurance_claim_no ILIKE $1 OR d.description ILIKE $1 OR d.damage_type ILIKE $1)

            UNION ALL

            SELECT 'expense'::text AS result_type, e.id, v.plate AS title,
                   concat_ws(' ', e.expense_type::text, e.invoice_number, e.note) AS subtitle,
                   e.payment_status AS status, e.created_at
            FROM expenses e
            JOIN vehicles v ON v.id = e.vehicle_id
            WHERE e.deleted_at IS NULL
              AND (v.plate ILIKE $1 OR e.invoice_number ILIKE $1 OR e.note ILIKE $1)

            UNION ALL

            SELECT 'task'::text AS result_type, t.id, t.task_type AS title,
                   concat_ws(' ', v.plate, t.description) AS subtitle,
                   t.task_status AS status, t.created_at
            FROM tasks t
            LEFT JOIN vehicles v ON v.id = t.related_vehicle_id
            WHERE t.deleted_at IS NULL
              AND (t.task_type ILIKE $1 OR t.description ILIKE $1 OR v.plate ILIKE $1)

            UNION ALL

            SELECT 'support_ticket'::text AS result_type, s.id, s.ticket_no AS title,
                   concat_ws(' ', s.title, s.reporter_name, s.reporter_phone) AS subtitle,
                   s.ticket_status AS status, s.created_at
            FROM support_tickets s
            WHERE s.deleted_at IS NULL
              AND (s.ticket_no ILIKE $1 OR s.title ILIKE $1 OR s.description ILIKE $1 OR s.reporter_name ILIKE $1 OR s.reporter_phone ILIKE $1)

            UNION ALL

            SELECT 'support_knowledge'::text AS result_type, kb.id, kb.title,
                   concat_ws(' ', kb.category, kb.problem) AS subtitle,
                   CASE WHEN kb.is_published THEN 'published' ELSE 'draft' END AS status,
                   kb.created_at
            FROM support_knowledge_base kb
            WHERE kb.deleted_at IS NULL
              AND (kb.title ILIKE $1 OR kb.problem ILIKE $1 OR kb.solution ILIKE $1 OR kb.tags::text ILIKE $1)
        ) results
        ORDER BY created_at DESC NULLS LAST
        LIMIT $2",
    )
    .bind(q)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}
