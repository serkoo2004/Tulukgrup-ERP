use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    notifications, AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{types::Json as SqlJson, FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/summary", get(get_support_summary))
        .route(
            "/knowledge-base",
            get(list_knowledge_base).post(create_knowledge_base),
        )
        .route(
            "/knowledge-base/:id",
            get(get_knowledge_base).patch(update_knowledge_base),
        )
        .route("/tickets", get(list_tickets).post(create_ticket))
        .route("/tickets/:id", get(get_ticket).patch(update_ticket))
        .route("/tickets/:id/events", get(list_ticket_events))
        .route("/tickets/:id/reply", post(reply_ticket_whatsapp))
}

#[derive(Debug, Deserialize)]
struct TicketQuery {
    ticket_status: Option<String>,
    priority: Option<String>,
    source_channel: Option<String>,
    assigned_user_id: Option<i64>,
    reporter_phone: Option<String>,
    q: Option<String>,
    overdue_only: Option<bool>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeBaseQuery {
    q: Option<String>,
    category: Option<String>,
    is_published: Option<bool>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeBaseCreate {
    title: String,
    category: Option<String>,
    problem: String,
    solution: String,
    tags: Option<Value>,
    source_ticket_id: Option<i64>,
    is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeBaseUpdate {
    title: Option<String>,
    category: Option<String>,
    problem: Option<String>,
    solution: Option<String>,
    tags: Option<Value>,
    is_published: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportSummary {
    total_count: i64,
    open_count: i64,
    in_progress_count: i64,
    waiting_user_count: i64,
    resolved_today_count: i64,
    closed_count: i64,
    critical_count: i64,
    whatsapp_open_count: i64,
    unassigned_count: i64,
    response_overdue_count: i64,
    resolution_overdue_count: i64,
    due_soon_count: i64,
    avg_satisfaction: Option<f64>,
    avg_resolution_hours: Option<f64>,
}

#[derive(Debug, FromRow, Serialize)]
struct KnowledgeBaseRow {
    id: i64,
    title: String,
    category: String,
    problem: String,
    solution: String,
    tags: Value,
    source_ticket_id: Option<i64>,
    source_ticket_no: Option<String>,
    is_published: bool,
    view_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TicketCreate {
    title: String,
    description: String,
    category: Option<String>,
    priority: Option<String>,
    source_channel: Option<String>,
    reporter_name: Option<String>,
    reporter_phone: Option<String>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TicketUpdate {
    title: Option<String>,
    description: Option<String>,
    category: Option<String>,
    priority: Option<String>,
    ticket_status: Option<String>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    resolution_note: Option<String>,
    satisfaction_score: Option<i32>,
    satisfaction_note: Option<String>,
    event_note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TicketReplyRequest {
    message: String,
    next_status: Option<String>,
    event_note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SupportTicketRow {
    pub id: i64,
    pub ticket_no: String,
    pub title: String,
    description: String,
    category: String,
    priority: String,
    ticket_status: String,
    source_channel: String,
    reporter_name: Option<String>,
    reporter_phone: Option<String>,
    reporter_user_id: Option<i64>,
    assigned_user_id: Option<i64>,
    assigned_user_name: Option<String>,
    assigned_department_id: Option<i64>,
    assigned_department_name: Option<String>,
    source_notification_id: Option<i64>,
    external_message_id: Option<String>,
    resolution_note: Option<String>,
    resolved_at: Option<DateTime<Utc>>,
    closed_at: Option<DateTime<Utc>>,
    sla_response_due_at: Option<DateTime<Utc>>,
    sla_resolution_due_at: Option<DateTime<Utc>>,
    first_response_at: Option<DateTime<Utc>>,
    escalation_level: i32,
    satisfaction_score: Option<i32>,
    satisfaction_note: Option<String>,
    sla_status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportTicketEventRow {
    id: i64,
    ticket_id: i64,
    event_type: String,
    note: Option<String>,
    old_status: Option<String>,
    new_status: Option<String>,
    created_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct SupportNotificationRow {
    id: i64,
    notification_type: String,
    receiver_user_id: Option<i64>,
    related_vehicle_id: Option<i64>,
    message: String,
    sent_via: String,
    delivery_status: String,
    recipient_phone: Option<String>,
    external_message_id: Option<String>,
    provider_payload: Option<Value>,
    delivery_error: Option<String>,
    sent_at: Option<DateTime<Utc>>,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Serialize)]
struct TicketReplyResponse {
    ticket: SupportTicketRow,
    notification: SupportNotificationRow,
}

pub struct WhatsAppTicketInput {
    pub reporter_phone: Option<String>,
    pub reporter_name: Option<String>,
    pub message: String,
    pub external_message_id: Option<String>,
}

async fn get_support_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<SupportSummary>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let summary = sqlx::query_as::<_, SupportSummary>(
        "SELECT
            count(*) AS total_count,
            count(*) FILTER (WHERE ticket_status = 'open') AS open_count,
            count(*) FILTER (WHERE ticket_status = 'in_progress') AS in_progress_count,
            count(*) FILTER (WHERE ticket_status = 'waiting_user') AS waiting_user_count,
            count(*) FILTER (WHERE resolved_at::date = CURRENT_DATE OR closed_at::date = CURRENT_DATE) AS resolved_today_count,
            count(*) FILTER (WHERE ticket_status IN ('resolved', 'closed')) AS closed_count,
            count(*) FILTER (WHERE priority = 'critical'::priority_level AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS critical_count,
            count(*) FILTER (WHERE source_channel = 'whatsapp' AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS whatsapp_open_count,
            count(*) FILTER (WHERE assigned_user_id IS NULL AND assigned_department_id IS NULL AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS unassigned_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'response_overdue') AS response_overdue_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'resolution_overdue') AS resolution_overdue_count,
            count(*) FILTER (WHERE support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) = 'due_soon') AS due_soon_count,
            avg(satisfaction_score::float8) FILTER (WHERE satisfaction_score IS NOT NULL) AS avg_satisfaction,
            (
              avg(EXTRACT(EPOCH FROM (COALESCE(resolved_at, closed_at) - created_at)) / 3600.0)
                FILTER (WHERE COALESCE(resolved_at, closed_at) IS NOT NULL)
            )::float8 AS avg_resolution_hours
         FROM support_tickets
         WHERE deleted_at IS NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(summary))
}

async fn list_knowledge_base(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<KnowledgeBaseQuery>,
) -> ApiResult<Json<Vec<KnowledgeBaseRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let q = query.q.map(|value| format!("%{}%", value));
    let rows = sqlx::query_as::<_, KnowledgeBaseRow>(
        "SELECT kb.id, kb.title, kb.category, kb.problem, kb.solution, kb.tags,
                kb.source_ticket_id, t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at, kb.created_by
         FROM support_knowledge_base kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id
         WHERE kb.deleted_at IS NULL
           AND ($1::text IS NULL OR kb.category = $1)
           AND ($2::boolean IS NULL OR kb.is_published = $2)
           AND ($3::text IS NULL OR kb.title ILIKE $3 OR kb.problem ILIKE $3 OR kb.solution ILIKE $3 OR kb.tags::text ILIKE $3)
         ORDER BY kb.is_published DESC, kb.updated_at DESC
         LIMIT $4",
    )
    .bind(query.category)
    .bind(query.is_published)
    .bind(q)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_knowledge_base(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<KnowledgeBaseCreate>,
) -> ApiResult<Json<KnowledgeBaseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_ticket_text(&payload.title, "title")?;
    validate_ticket_text(&payload.problem, "problem")?;
    validate_ticket_text(&payload.solution, "solution")?;
    if let Some(ticket_id) = payload.source_ticket_id {
        find_ticket(&state.pool, ticket_id).await?;
    }

    let row = sqlx::query_as::<_, KnowledgeBaseRow>(
        "WITH inserted AS (
            INSERT INTO support_knowledge_base
              (title, category, problem, solution, tags, source_ticket_id, is_published, created_by)
            VALUES ($1, COALESCE($2, 'it_support'), $3, $4, COALESCE($5, '[]'::jsonb), $6, COALESCE($7, true), $8)
            RETURNING *
         )
         SELECT kb.id, kb.title, kb.category, kb.problem, kb.solution, kb.tags,
                kb.source_ticket_id, t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at, kb.created_by
         FROM inserted kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id",
    )
    .bind(payload.title)
    .bind(payload.category)
    .bind(payload.problem)
    .bind(payload.solution)
    .bind(payload.tags.map(SqlJson))
    .bind(payload.source_ticket_id)
    .bind(payload.is_published)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    if let Some(ticket_id) = row.source_ticket_id {
        insert_event(
            &state.pool,
            ticket_id,
            "knowledge_base_created",
            Some("Çözüm bilgi bankasına eklendi"),
            None,
            None,
            Some(current.id),
        )
        .await?;
    }
    audit::write_audit(
        &state.pool,
        "support_knowledge_base",
        Some(row.id),
        "create",
        None,
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;
    Ok(Json(row))
}

async fn get_knowledge_base(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<KnowledgeBaseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let row = sqlx::query_as::<_, KnowledgeBaseRow>(
        "WITH updated AS (
            UPDATE support_knowledge_base
            SET view_count = view_count + 1,
                updated_at = updated_at
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
         )
         SELECT kb.id, kb.title, kb.category, kb.problem, kb.solution, kb.tags,
                kb.source_ticket_id, t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at, kb.created_by
         FROM updated kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(row))
}

async fn update_knowledge_base(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<KnowledgeBaseUpdate>,
) -> ApiResult<Json<KnowledgeBaseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    if let Some(title) = payload.title.as_deref() {
        validate_ticket_text(title, "title")?;
    }
    if let Some(problem) = payload.problem.as_deref() {
        validate_ticket_text(problem, "problem")?;
    }
    if let Some(solution) = payload.solution.as_deref() {
        validate_ticket_text(solution, "solution")?;
    }
    let old = find_knowledge_base(&state.pool, id).await?;
    let row = sqlx::query_as::<_, KnowledgeBaseRow>(
        "WITH updated AS (
            UPDATE support_knowledge_base
            SET title = COALESCE($2, title),
                category = COALESCE($3, category),
                problem = COALESCE($4, problem),
                solution = COALESCE($5, solution),
                tags = COALESCE($6, tags),
                is_published = COALESCE($7, is_published),
                updated_by = $8,
                updated_at = now()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
         )
         SELECT kb.id, kb.title, kb.category, kb.problem, kb.solution, kb.tags,
                kb.source_ticket_id, t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at, kb.created_by
         FROM updated kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id",
    )
    .bind(id)
    .bind(payload.title)
    .bind(payload.category)
    .bind(payload.problem)
    .bind(payload.solution)
    .bind(payload.tags.map(SqlJson))
    .bind(payload.is_published)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "support_knowledge_base",
        Some(row.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;
    Ok(Json(row))
}

async fn list_tickets(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TicketQuery>,
) -> ApiResult<Json<Vec<SupportTicketRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    let can_view_all = matches!(current.role.as_str(), "admin" | "manager" | "operation");
    let q = query.q.map(|value| format!("%{}%", value));
    let rows = sqlx::query_as::<_, SupportTicketRow>(
        "SELECT t.id, t.ticket_no, t.title, t.description, t.category, t.priority::text AS priority,
                t.ticket_status, t.source_channel, t.reporter_name, t.reporter_phone, t.reporter_user_id,
                t.assigned_user_id, u.full_name AS assigned_user_name,
                t.assigned_department_id, d.name AS assigned_department_name,
                t.source_notification_id, t.external_message_id, t.resolution_note,
                t.resolved_at, t.closed_at, t.sla_response_due_at, t.sla_resolution_due_at,
                t.first_response_at, t.escalation_level, t.satisfaction_score, t.satisfaction_note,
                support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at) AS sla_status,
                t.created_at, t.updated_at, t.created_by
         FROM support_tickets t
         LEFT JOIN users u ON u.id = t.assigned_user_id
         LEFT JOIN departments d ON d.id = t.assigned_department_id
         WHERE t.deleted_at IS NULL
           AND ($1::boolean = true OR t.reporter_user_id = $2 OR t.assigned_user_id = $2)
           AND ($3::text IS NULL OR t.ticket_status = $3)
           AND ($4::text IS NULL OR t.priority::text = $4)
           AND ($5::text IS NULL OR t.source_channel = $5)
           AND ($6::bigint IS NULL OR t.assigned_user_id = $6)
           AND ($7::text IS NULL OR t.reporter_phone = $7)
           AND ($8::text IS NULL OR t.ticket_no ILIKE $8 OR t.title ILIKE $8 OR t.description ILIKE $8 OR t.reporter_name ILIKE $8 OR t.reporter_phone ILIKE $8)
           AND ($9::boolean = false OR support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at) IN ('response_overdue', 'resolution_overdue'))
         ORDER BY
           CASE support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at)
             WHEN 'resolution_overdue' THEN 1
             WHEN 'response_overdue' THEN 2
             WHEN 'due_soon' THEN 3
             ELSE 4
           END,
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
         LIMIT $10",
    )
    .bind(can_view_all)
    .bind(current.id)
    .bind(query.ticket_status)
    .bind(query.priority)
    .bind(query.source_channel)
    .bind(query.assigned_user_id)
    .bind(query.reporter_phone)
    .bind(q)
    .bind(query.overdue_only.unwrap_or(false))
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TicketCreate>,
) -> ApiResult<Json<SupportTicketRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles_or_mobile_permission(
        &state.pool,
        &current,
        &["admin", "manager", "operation"],
        "support_ticket",
        "create",
    )
    .await?;
    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    validate_priority(&priority)?;
    let source_channel = payload.source_channel.unwrap_or_else(|| "web".to_string());
    validate_source_channel(&source_channel)?;
    let category = payload.category.unwrap_or_else(|| "it_support".to_string());
    validate_ticket_text(&payload.title, "title")?;
    validate_ticket_text(&payload.description, "description")?;

    let row = insert_ticket(
        &state.pool,
        TicketInsert {
            title: payload.title,
            description: payload.description,
            category,
            priority,
            source_channel,
            reporter_name: payload.reporter_name,
            reporter_phone: payload.reporter_phone,
            reporter_user_id: Some(current.id),
            assigned_user_id: payload.assigned_user_id,
            assigned_department_id: payload.assigned_department_id,
            source_notification_id: None,
            external_message_id: None,
            created_by: Some(current.id),
        },
    )
    .await?;
    insert_event(
        &state.pool,
        row.id,
        "created",
        Some("Destek talebi oluşturuldu"),
        None,
        Some(&row.ticket_status),
        Some(current.id),
    )
    .await?;
    audit::write_audit(
        &state.pool,
        "support_tickets",
        Some(row.id),
        "create",
        None,
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;
    Ok(Json(row))
}

async fn get_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<SupportTicketRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let row = find_ticket(&state.pool, id).await?;
    if can_access_ticket(&current, &row) {
        Ok(Json(row))
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn update_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<TicketUpdate>,
) -> ApiResult<Json<SupportTicketRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let old = find_ticket(&state.pool, id).await?;
    if !can_access_ticket(&current, &old) {
        return Err(ApiError::Forbidden);
    }
    if old.reporter_user_id == Some(current.id)
        && !matches!(current.role.as_str(), "admin" | "manager" | "operation")
    {
        if payload.assigned_user_id.is_some()
            || payload.assigned_department_id.is_some()
            || payload.resolution_note.is_some()
        {
            return Err(ApiError::Forbidden);
        }
    }
    if let Some(priority) = payload.priority.as_deref() {
        validate_priority(priority)?;
    }
    if let Some(status) = payload.ticket_status.as_deref() {
        validate_ticket_status(status)?;
    }
    if let Some(title) = payload.title.as_deref() {
        validate_ticket_text(title, "title")?;
    }
    if let Some(description) = payload.description.as_deref() {
        validate_ticket_text(description, "description")?;
    }
    if let Some(score) = payload.satisfaction_score {
        validate_satisfaction_score(score)?;
    }

    let row = sqlx::query_as::<_, SupportTicketRow>(
        "UPDATE support_tickets
         SET title = COALESCE($2, title),
             description = COALESCE($3, description),
             category = COALESCE($4, category),
             priority = COALESCE($5::priority_level, priority),
             ticket_status = COALESCE($6, ticket_status),
             assigned_user_id = COALESCE($7, assigned_user_id),
             assigned_department_id = COALESCE($8, assigned_department_id),
             resolution_note = COALESCE($9, resolution_note),
             satisfaction_score = COALESCE($10, satisfaction_score),
             satisfaction_note = COALESCE($11, satisfaction_note),
             first_response_at = CASE
               WHEN first_response_at IS NULL
                AND (($6 IS NOT NULL AND $6 != 'open') OR $7 IS NOT NULL OR $8 IS NOT NULL OR $9 IS NOT NULL)
               THEN now()
               ELSE first_response_at
             END,
             resolved_at = CASE WHEN $6 = 'resolved' THEN COALESCE(resolved_at, now()) ELSE resolved_at END,
             closed_at = CASE WHEN $6 = 'closed' THEN COALESCE(closed_at, now()) ELSE closed_at END,
             escalation_level = CASE
               WHEN ticket_status NOT IN ('resolved', 'closed', 'cancelled')
                AND sla_resolution_due_at IS NOT NULL
                AND sla_resolution_due_at < now()
               THEN GREATEST(escalation_level, 1)
               ELSE escalation_level
             END,
             updated_by = $12,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, ticket_no, title, description, category, priority::text AS priority,
                   ticket_status, source_channel, reporter_name, reporter_phone, reporter_user_id,
                   assigned_user_id, (SELECT full_name FROM users WHERE id = assigned_user_id) AS assigned_user_name,
                   assigned_department_id, (SELECT name FROM departments WHERE id = assigned_department_id) AS assigned_department_name,
                   source_notification_id, external_message_id, resolution_note, resolved_at, closed_at,
                   sla_response_due_at, sla_resolution_due_at, first_response_at, escalation_level,
                   satisfaction_score, satisfaction_note,
                   support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) AS sla_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.category)
    .bind(payload.priority)
    .bind(payload.ticket_status.clone())
    .bind(payload.assigned_user_id)
    .bind(payload.assigned_department_id)
    .bind(payload.resolution_note)
    .bind(payload.satisfaction_score)
    .bind(payload.satisfaction_note)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    insert_event(
        &state.pool,
        row.id,
        if old.ticket_status != row.ticket_status {
            "status_changed"
        } else {
            "updated"
        },
        payload
            .event_note
            .as_deref()
            .or(Some("Destek talebi güncellendi")),
        Some(&old.ticket_status),
        Some(&row.ticket_status),
        Some(current.id),
    )
    .await?;
    audit::write_audit(
        &state.pool,
        "support_tickets",
        Some(row.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;
    Ok(Json(row))
}

async fn reply_ticket_whatsapp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<TicketReplyRequest>,
) -> ApiResult<Json<TicketReplyResponse>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let old = find_ticket(&state.pool, id).await?;
    if !can_access_ticket(&current, &old) {
        return Err(ApiError::Forbidden);
    }
    validate_ticket_text(&payload.message, "message")?;
    let next_status = payload
        .next_status
        .unwrap_or_else(|| "waiting_user".to_string());
    validate_ticket_status(&next_status)?;
    if matches!(next_status.as_str(), "closed" | "cancelled") {
        return Err(ApiError::BadRequest(
            "reply next_status cannot be closed or cancelled".to_string(),
        ));
    }
    let recipient_phone = old
        .reporter_phone
        .clone()
        .filter(|phone| !phone.trim().is_empty())
        .ok_or_else(|| {
            ApiError::BadRequest("reporter_phone is required for whatsapp reply".to_string())
        })?;
    let notification_type = "it_support_ticket_reply";
    let (provider_payload, delivery_status, delivery_error, external_message_id) =
        notifications::dispatch_whatsapp_message(
            notification_type,
            &recipient_phone,
            &payload.message,
        )
        .await;

    let notification = sqlx::query_as::<_, SupportNotificationRow>(
        "INSERT INTO notifications
         (notification_type, message, sent_via, delivery_status, recipient_phone,
          provider_payload, delivery_error, external_message_id, sent_at, created_by)
         VALUES ($1, $2, $3::notification_channel, $4, $5, $6, $7, $8,
                 CASE WHEN $4 = 'sent' THEN now() ELSE NULL END, $9)
         RETURNING id, notification_type, receiver_user_id, related_vehicle_id, message,
                   sent_via::text AS sent_via, delivery_status, recipient_phone,
                   external_message_id, provider_payload, delivery_error, sent_at, read_at,
                   created_at, updated_at, created_by",
    )
    .bind(notification_type)
    .bind(&payload.message)
    .bind("whatsapp")
    .bind(&delivery_status)
    .bind(&recipient_phone)
    .bind(provider_payload)
    .bind(delivery_error)
    .bind(external_message_id)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    let ticket = sqlx::query_as::<_, SupportTicketRow>(
        "UPDATE support_tickets
         SET ticket_status = $2,
             first_response_at = COALESCE(first_response_at, now()),
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, ticket_no, title, description, category, priority::text AS priority,
                   ticket_status, source_channel, reporter_name, reporter_phone, reporter_user_id,
                   assigned_user_id, (SELECT full_name FROM users WHERE id = assigned_user_id) AS assigned_user_name,
                   assigned_department_id, (SELECT name FROM departments WHERE id = assigned_department_id) AS assigned_department_name,
                   source_notification_id, external_message_id, resolution_note, resolved_at, closed_at,
                   sla_response_due_at, sla_resolution_due_at, first_response_at, escalation_level,
                   satisfaction_score, satisfaction_note,
                   support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) AS sla_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(&next_status)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let event_note = payload.event_note.as_deref().unwrap_or(&payload.message);
    insert_event(
        &state.pool,
        ticket.id,
        "whatsapp_reply",
        Some(event_note),
        Some(&old.ticket_status),
        Some(&ticket.ticket_status),
        Some(current.id),
    )
    .await?;
    audit::write_audit(
        &state.pool,
        "support_tickets",
        Some(ticket.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        json!({ "ticket": ticket, "notification": notification }),
        Some(current.id),
        &headers,
    )
    .await?;

    let ticket = find_ticket(&state.pool, id).await?;
    Ok(Json(TicketReplyResponse {
        ticket,
        notification,
    }))
}

async fn list_ticket_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<SupportTicketEventRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    let ticket = find_ticket(&state.pool, id).await?;
    if !can_access_ticket(&current, &ticket) {
        return Err(ApiError::Forbidden);
    }
    let rows = sqlx::query_as::<_, SupportTicketEventRow>(
        "SELECT id, ticket_id, event_type, note, old_status, new_status, created_at, created_by
         FROM support_ticket_events
         WHERE ticket_id = $1
         ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_ticket_from_whatsapp_message(
    pool: &PgPool,
    input: WhatsAppTicketInput,
) -> ApiResult<SupportTicketRow> {
    validate_ticket_text(&input.message, "message")?;
    if let Some(message_id) = input.external_message_id.as_deref() {
        if let Some(existing) = find_ticket_by_external_message(pool, message_id).await? {
            return Ok(existing);
        }
    }
    let title = whatsapp_title(&input.message);
    let row = insert_ticket(
        pool,
        TicketInsert {
            title,
            description: input.message,
            category: "it_support".to_string(),
            priority: "medium".to_string(),
            source_channel: "whatsapp".to_string(),
            reporter_name: input.reporter_name,
            reporter_phone: input.reporter_phone,
            reporter_user_id: None,
            assigned_user_id: None,
            assigned_department_id: None,
            source_notification_id: None,
            external_message_id: input.external_message_id,
            created_by: None,
        },
    )
    .await?;
    insert_event(
        pool,
        row.id,
        "created_from_whatsapp",
        Some("WhatsApp mesajından destek talebi oluşturuldu"),
        None,
        Some(&row.ticket_status),
        None,
    )
    .await?;
    sqlx::query(
        "INSERT INTO notifications (notification_type, message, sent_via, delivery_status)
         VALUES ('it_support_ticket_created', $1, 'system', 'pending')",
    )
    .bind(format!(
        "Yeni IT destek talebi {}: {}",
        row.ticket_no, row.title
    ))
    .execute(pool)
    .await?;
    Ok(row)
}

struct TicketInsert {
    title: String,
    description: String,
    category: String,
    priority: String,
    source_channel: String,
    reporter_name: Option<String>,
    reporter_phone: Option<String>,
    reporter_user_id: Option<i64>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    source_notification_id: Option<i64>,
    external_message_id: Option<String>,
    created_by: Option<i64>,
}

async fn insert_ticket(pool: &PgPool, input: TicketInsert) -> ApiResult<SupportTicketRow> {
    sqlx::query_as::<_, SupportTicketRow>(
        "WITH inserted AS (
            INSERT INTO support_tickets
             (ticket_no, title, description, category, priority, source_channel,
              reporter_name, reporter_phone, reporter_user_id, assigned_user_id,
              assigned_department_id, source_notification_id, external_message_id, created_by,
              sla_response_due_at, sla_resolution_due_at)
             VALUES (
               'IT-' || to_char(now(), 'YYYYMMDD') || '-' || upper(substr(md5(random()::text || clock_timestamp()::text), 1, 6)),
               $1, $2, $3, $4::priority_level, $5, $6, $7, $8, $9, $10, $11, $12, $13,
               now() + CASE $4::text
                 WHEN 'critical' THEN INTERVAL '1 hour'
                 WHEN 'high' THEN INTERVAL '4 hours'
                 WHEN 'medium' THEN INTERVAL '8 hours'
                 ELSE INTERVAL '24 hours'
               END,
               now() + CASE $4::text
                 WHEN 'critical' THEN INTERVAL '4 hours'
                 WHEN 'high' THEN INTERVAL '24 hours'
                 WHEN 'medium' THEN INTERVAL '72 hours'
                 ELSE INTERVAL '120 hours'
               END
             )
             RETURNING *
         )
         SELECT i.id, i.ticket_no, i.title, i.description, i.category, i.priority::text AS priority,
                i.ticket_status, i.source_channel, i.reporter_name, i.reporter_phone, i.reporter_user_id,
                i.assigned_user_id, u.full_name AS assigned_user_name,
                i.assigned_department_id, d.name AS assigned_department_name,
                i.source_notification_id, i.external_message_id, i.resolution_note,
                i.resolved_at, i.closed_at, i.sla_response_due_at, i.sla_resolution_due_at,
                i.first_response_at, i.escalation_level, i.satisfaction_score, i.satisfaction_note,
                support_sla_status(i.ticket_status, i.first_response_at, i.sla_response_due_at, i.sla_resolution_due_at) AS sla_status,
                i.created_at, i.updated_at, i.created_by
         FROM inserted i
         LEFT JOIN users u ON u.id = i.assigned_user_id
         LEFT JOIN departments d ON d.id = i.assigned_department_id",
    )
    .bind(input.title)
    .bind(input.description)
    .bind(input.category)
    .bind(input.priority)
    .bind(input.source_channel)
    .bind(input.reporter_name)
    .bind(input.reporter_phone)
    .bind(input.reporter_user_id)
    .bind(input.assigned_user_id)
    .bind(input.assigned_department_id)
    .bind(input.source_notification_id)
    .bind(input.external_message_id)
    .bind(input.created_by)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)
}

async fn find_ticket(pool: &PgPool, id: i64) -> ApiResult<SupportTicketRow> {
    sqlx::query_as::<_, SupportTicketRow>(
        "SELECT t.id, t.ticket_no, t.title, t.description, t.category, t.priority::text AS priority,
                t.ticket_status, t.source_channel, t.reporter_name, t.reporter_phone, t.reporter_user_id,
                t.assigned_user_id, u.full_name AS assigned_user_name,
                t.assigned_department_id, d.name AS assigned_department_name,
                t.source_notification_id, t.external_message_id, t.resolution_note,
                t.resolved_at, t.closed_at, t.sla_response_due_at, t.sla_resolution_due_at,
                t.first_response_at, t.escalation_level, t.satisfaction_score, t.satisfaction_note,
                support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at) AS sla_status,
                t.created_at, t.updated_at, t.created_by
         FROM support_tickets t
         LEFT JOIN users u ON u.id = t.assigned_user_id
         LEFT JOIN departments d ON d.id = t.assigned_department_id
         WHERE t.id = $1 AND t.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_ticket_by_external_message(
    pool: &PgPool,
    external_message_id: &str,
) -> ApiResult<Option<SupportTicketRow>> {
    sqlx::query_as::<_, SupportTicketRow>(
        "SELECT t.id, t.ticket_no, t.title, t.description, t.category, t.priority::text AS priority,
                t.ticket_status, t.source_channel, t.reporter_name, t.reporter_phone, t.reporter_user_id,
                t.assigned_user_id, u.full_name AS assigned_user_name,
                t.assigned_department_id, d.name AS assigned_department_name,
                t.source_notification_id, t.external_message_id, t.resolution_note,
                t.resolved_at, t.closed_at, t.sla_response_due_at, t.sla_resolution_due_at,
                t.first_response_at, t.escalation_level, t.satisfaction_score, t.satisfaction_note,
                support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at) AS sla_status,
                t.created_at, t.updated_at, t.created_by
         FROM support_tickets t
         LEFT JOIN users u ON u.id = t.assigned_user_id
         LEFT JOIN departments d ON d.id = t.assigned_department_id
         WHERE t.external_message_id = $1 AND t.deleted_at IS NULL",
    )
    .bind(external_message_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)
}

async fn find_knowledge_base(pool: &PgPool, id: i64) -> ApiResult<KnowledgeBaseRow> {
    sqlx::query_as::<_, KnowledgeBaseRow>(
        "SELECT kb.id, kb.title, kb.category, kb.problem, kb.solution, kb.tags,
                kb.source_ticket_id, t.ticket_no AS source_ticket_no,
                kb.is_published, kb.view_count, kb.created_at, kb.updated_at, kb.created_by
         FROM support_knowledge_base kb
         LEFT JOIN support_tickets t ON t.id = kb.source_ticket_id
         WHERE kb.id = $1 AND kb.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn insert_event(
    pool: &PgPool,
    ticket_id: i64,
    event_type: &str,
    note: Option<&str>,
    old_status: Option<&str>,
    new_status: Option<&str>,
    created_by: Option<i64>,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO support_ticket_events (ticket_id, event_type, note, old_status, new_status, created_by)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(ticket_id)
    .bind(event_type)
    .bind(note)
    .bind(old_status)
    .bind(new_status)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

fn can_access_ticket(user: &auth::AuthUser, ticket: &SupportTicketRow) -> bool {
    matches!(user.role.as_str(), "admin" | "manager" | "operation")
        || ticket.reporter_user_id == Some(user.id)
        || ticket.assigned_user_id == Some(user.id)
}

fn whatsapp_title(message: &str) -> String {
    let compact = message.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = compact.chars().take(80).collect::<String>();
    if title.is_empty() {
        "WhatsApp destek talebi".to_string()
    } else {
        title
    }
}

fn validate_ticket_text(value: &str, field: &str) -> ApiResult<()> {
    if value.trim().is_empty() {
        return Err(ApiError::BadRequest(format!("{} is required", field)));
    }
    Ok(())
}

fn validate_priority(priority: &str) -> ApiResult<()> {
    match priority {
        "low" | "medium" | "high" | "critical" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid ticket priority".to_string())),
    }
}

fn validate_ticket_status(status: &str) -> ApiResult<()> {
    match status {
        "open" | "in_progress" | "waiting_user" | "resolved" | "closed" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid ticket status".to_string())),
    }
}

fn validate_source_channel(source_channel: &str) -> ApiResult<()> {
    match source_channel {
        "web" | "mobile" | "whatsapp" | "system" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid support source channel".to_string(),
        )),
    }
}

fn validate_satisfaction_score(score: i32) -> ApiResult<()> {
    if (1..=5).contains(&score) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "satisfaction_score must be between 1 and 5".to_string(),
        ))
    }
}
