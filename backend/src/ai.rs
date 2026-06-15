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
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{types::Json as SqlJson, FromRow};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/capabilities", get(list_ai_capabilities))
        .route("/provider", get(get_ai_provider_status))
        .route("/prompt-templates", get(list_prompt_templates))
        .route("/recommendations", get(list_ai_recommendations))
        .route("/jobs", get(list_ai_jobs).post(create_ai_job))
        .route("/jobs/:id", get(get_ai_job).patch(update_ai_job))
        .route("/jobs/:id/approve", post(approve_ai_job))
        .route("/jobs/:id/task-draft", get(get_ai_task_draft))
        .route(
            "/jobs/:id/notification-draft",
            get(get_ai_notification_draft),
        )
        .route("/vehicles/:id/context", get(get_vehicle_ai_context))
        .route("/vehicles/:id/analyze", post(create_vehicle_ai_analysis))
        .route(
            "/support/tickets/:id/context",
            get(get_support_ticket_ai_context),
        )
        .route(
            "/support/tickets/:id/analyze",
            post(create_support_ticket_ai_analysis),
        )
}

#[derive(Debug, Deserialize)]
struct AiJobQuery {
    analysis_type: Option<String>,
    related_vehicle_id: Option<i64>,
    job_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AiJobCreate {
    analysis_type: String,
    related_vehicle_id: Option<i64>,
    source_table: Option<String>,
    source_record_id: Option<i64>,
    input_data: Option<Value>,
    requires_human_approval: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct VehicleAiAnalysisCreate {
    analysis_type: String,
    requires_human_approval: Option<bool>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SupportTicketAiAnalysisCreate {
    analysis_type: String,
    requires_human_approval: Option<bool>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AiJobUpdate {
    job_status: String,
    result_data: Option<Value>,
    confidence_score: Option<Decimal>,
}

#[derive(Debug, Serialize)]
struct AiCapability {
    analysis_type: &'static str,
    title: &'static str,
    target_modules: Vec<&'static str>,
    human_approval_required: bool,
    description: &'static str,
}

#[derive(Debug, Serialize)]
struct AiProviderStatus {
    provider: String,
    model: String,
    configured: bool,
    execution_mode: String,
    base_url_configured: bool,
    api_key_configured: bool,
    timeout_seconds: u64,
    supports_external_models: bool,
    supports_local_models: bool,
    required_env: Vec<&'static str>,
    missing_env: Vec<&'static str>,
    next_step: String,
}

#[derive(Debug, Serialize)]
struct AiPromptTemplate {
    analysis_type: &'static str,
    system_goal: &'static str,
    required_context: Vec<&'static str>,
    output_contract: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct AiTaskDraft {
    task_type: String,
    related_vehicle_id: Option<i64>,
    priority: String,
    description: String,
    source_ai_job_id: i64,
}

#[derive(Debug, Serialize)]
struct AiNotificationDraft {
    notification_type: String,
    related_vehicle_id: Option<i64>,
    sent_via: String,
    delivery_status: String,
    message: String,
    source_ai_job_id: i64,
}

#[derive(Debug, Serialize)]
struct AiRecommendation {
    recommendation_type: String,
    severity: String,
    title: String,
    detail: String,
    source_module: String,
    source_count: i64,
    suggested_analysis_type: String,
}

#[derive(Debug, FromRow)]
struct RecommendationSignals {
    overdue_tasks: i64,
    critical_tasks: i64,
    expiring_policies: i64,
    ended_policies: i64,
    open_damages: i64,
    open_value_loss_claims: i64,
    suspicious_km_logs: i64,
    failed_import_jobs: i64,
    open_support_tickets: i64,
    overdue_support_tickets: i64,
    critical_support_tickets: i64,
    whatsapp_support_tickets: i64,
}

#[derive(Debug, FromRow, Serialize)]
struct AiJobRow {
    id: i64,
    analysis_type: String,
    related_vehicle_id: Option<i64>,
    source_table: Option<String>,
    source_record_id: Option<i64>,
    input_data: Option<Value>,
    result_data: Option<Value>,
    confidence_score: Option<Decimal>,
    job_status: String,
    requires_human_approval: bool,
    approved_by: Option<i64>,
    approved_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

async fn list_ai_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<AiCapability>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(ai_capabilities()))
}

async fn get_ai_provider_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<AiProviderStatus>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let provider = state.config.ai_provider.trim().to_ascii_lowercase();
    let api_key_configured = state.config.ai_api_key.is_some();
    let base_url_configured = state.config.ai_base_url.is_some();
    let required_env = required_ai_env(&provider);
    let missing_env = missing_ai_env(&provider, api_key_configured, base_url_configured);
    let configured = provider != "disabled" && missing_env.is_empty();
    Ok(Json(AiProviderStatus {
        provider: state.config.ai_provider.clone(),
        model: state.config.ai_model.clone(),
        configured,
        execution_mode: if configured {
            "worker_ready".to_string()
        } else {
            "job_contract_only".to_string()
        },
        base_url_configured,
        api_key_configured,
        timeout_seconds: state.config.ai_timeout_seconds,
        supports_external_models: true,
        supports_local_models: true,
        required_env,
        missing_env,
        next_step: ai_next_step(&provider, configured),
    }))
}

async fn list_prompt_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<AiPromptTemplate>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(prompt_templates()))
}

async fn list_ai_recommendations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<AiRecommendation>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let signals = sqlx::query_as::<_, RecommendationSignals>(
        "SELECT
            (SELECT count(*) FROM tasks WHERE deleted_at IS NULL AND due_date < CURRENT_DATE AND task_status NOT IN ('completed', 'cancelled')) AS overdue_tasks,
            (SELECT count(*) FROM tasks WHERE deleted_at IS NULL AND priority = 'critical'::priority_level AND task_status NOT IN ('completed', 'cancelled')) AS critical_tasks,
            (SELECT count(*) FROM insurance_policies WHERE deleted_at IS NULL AND end_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '30 days') AS expiring_policies,
            (SELECT count(*) FROM insurance_policies WHERE deleted_at IS NULL AND (renewal_status = 'ended'::renewal_status OR end_date < CURRENT_DATE)) AS ended_policies,
            (SELECT count(*) FROM damages WHERE deleted_at IS NULL AND damage_status IN ('open', 'expertise', 'insurance')) AS open_damages,
            (SELECT count(*) FROM value_loss_claims WHERE deleted_at IS NULL AND claim_status NOT IN ('closed', 'cancelled', 'paid')) AS open_value_loss_claims,
            (SELECT count(*) FROM km_logs WHERE deleted_at IS NULL AND verification_status = 'suspicious'::verification_status) AS suspicious_km_logs,
            (SELECT count(*) FROM import_jobs WHERE deleted_at IS NULL AND import_status = 'failed') AS failed_import_jobs,
            (SELECT count(*) FROM support_tickets WHERE deleted_at IS NULL AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS open_support_tickets,
            (SELECT count(*) FROM support_tickets WHERE deleted_at IS NULL AND support_sla_status(ticket_status, first_response_at, sla_response_due_at, sla_resolution_due_at) IN ('response_overdue', 'resolution_overdue')) AS overdue_support_tickets,
            (SELECT count(*) FROM support_tickets WHERE deleted_at IS NULL AND priority = 'critical'::priority_level AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS critical_support_tickets,
            (SELECT count(*) FROM support_tickets WHERE deleted_at IS NULL AND source_channel = 'whatsapp' AND ticket_status NOT IN ('resolved', 'closed', 'cancelled')) AS whatsapp_support_tickets",
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(build_recommendations(signals)))
}

async fn list_ai_jobs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AiJobQuery>,
) -> ApiResult<Json<Vec<AiJobRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = sqlx::query_as::<_, AiJobRow>(
        "SELECT id, analysis_type, related_vehicle_id, source_table, source_record_id,
                input_data, result_data, confidence_score, job_status, requires_human_approval,
                approved_by, approved_at, created_at, updated_at, created_by
         FROM ai_analysis_jobs
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR analysis_type = $1)
           AND ($2::bigint IS NULL OR related_vehicle_id = $2)
           AND ($3::text IS NULL OR job_status = $3)
         ORDER BY created_at DESC
         LIMIT $4",
    )
    .bind(query.analysis_type)
    .bind(query.related_vehicle_id)
    .bind(query.job_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_ai_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AiJobCreate>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_analysis_type(&payload.analysis_type)?;

    let row = sqlx::query_as::<_, AiJobRow>(
        "INSERT INTO ai_analysis_jobs
         (analysis_type, related_vehicle_id, source_table, source_record_id,
          input_data, requires_human_approval, created_by)
         VALUES ($1, $2, $3, $4, $5, COALESCE($6, true), $7)
         RETURNING id, analysis_type, related_vehicle_id, source_table, source_record_id,
                   input_data, result_data, confidence_score, job_status, requires_human_approval,
                   approved_by, approved_at, created_at, updated_at, created_by",
    )
    .bind(payload.analysis_type)
    .bind(payload.related_vehicle_id)
    .bind(payload.source_table)
    .bind(payload.source_record_id)
    .bind(payload.input_data.map(SqlJson))
    .bind(payload.requires_human_approval)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "ai_analysis_jobs",
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

async fn get_ai_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(find_ai_job(&state.pool, id).await?))
}

async fn update_ai_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<AiJobUpdate>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_job_status(&payload.job_status)?;
    let old = find_ai_job(&state.pool, id).await?;

    let row = sqlx::query_as::<_, AiJobRow>(
        "UPDATE ai_analysis_jobs
         SET job_status = $2,
             result_data = COALESCE($3, result_data),
             confidence_score = COALESCE($4, confidence_score),
             updated_by = $5,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, analysis_type, related_vehicle_id, source_table, source_record_id,
                   input_data, result_data, confidence_score, job_status, requires_human_approval,
                   approved_by, approved_at, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.job_status)
    .bind(payload.result_data.map(SqlJson))
    .bind(payload.confidence_score)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "ai_analysis_jobs",
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

async fn approve_ai_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_ai_job(&state.pool, id).await?;

    let row = sqlx::query_as::<_, AiJobRow>(
        "UPDATE ai_analysis_jobs
         SET approved_by = $2,
             approved_at = now(),
             job_status = CASE WHEN job_status = 'completed' THEN job_status ELSE 'approved' END,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, analysis_type, related_vehicle_id, source_table, source_record_id,
                   input_data, result_data, confidence_score, job_status, requires_human_approval,
                   approved_by, approved_at, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "ai_analysis_jobs",
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

async fn get_ai_task_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<AiTaskDraft>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let job = find_ai_job(&state.pool, id).await?;
    Ok(Json(build_task_draft(&job)))
}

async fn get_ai_notification_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<AiNotificationDraft>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let job = find_ai_job(&state.pool, id).await?;
    Ok(Json(build_notification_draft(&job)))
}

async fn get_vehicle_ai_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(build_vehicle_context(&state.pool, id).await?))
}

async fn create_vehicle_ai_analysis(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<VehicleAiAnalysisCreate>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_analysis_type(&payload.analysis_type)?;
    let context = build_vehicle_context(&state.pool, id).await?;
    let input_data = json!({
        "context": context,
        "operator_note": payload.note,
        "safety_contract": {
            "no_automatic_critical_action": true,
            "requires_human_approval_for_status_or_financial_changes": true,
            "source": "backend_vehicle_context"
        }
    });

    let row = sqlx::query_as::<_, AiJobRow>(
        "INSERT INTO ai_analysis_jobs
         (analysis_type, related_vehicle_id, source_table, source_record_id,
          input_data, requires_human_approval, created_by)
         VALUES ($1, $2, 'vehicles', $2, $3, COALESCE($4, true), $5)
         RETURNING id, analysis_type, related_vehicle_id, source_table, source_record_id,
                   input_data, result_data, confidence_score, job_status, requires_human_approval,
                   approved_by, approved_at, created_at, updated_at, created_by",
    )
    .bind(payload.analysis_type)
    .bind(id)
    .bind(SqlJson(input_data))
    .bind(payload.requires_human_approval)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "ai_analysis_jobs",
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

async fn get_support_ticket_ai_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(build_support_ticket_context(&state.pool, id).await?))
}

async fn create_support_ticket_ai_analysis(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<SupportTicketAiAnalysisCreate>,
) -> ApiResult<Json<AiJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_analysis_type(&payload.analysis_type)?;
    let context = build_support_ticket_context(&state.pool, id).await?;
    let input_data = json!({
        "context": context,
        "operator_note": payload.note,
        "safety_contract": {
            "no_automatic_priority_or_status_change": true,
            "requires_human_approval_for_customer_message": true,
            "source": "backend_support_ticket_context"
        }
    });

    let row = sqlx::query_as::<_, AiJobRow>(
        "INSERT INTO ai_analysis_jobs
         (analysis_type, source_table, source_record_id, input_data, requires_human_approval, created_by)
         VALUES ($1, 'support_tickets', $2, $3, COALESCE($4, true), $5)
         RETURNING id, analysis_type, related_vehicle_id, source_table, source_record_id,
                   input_data, result_data, confidence_score, job_status, requires_human_approval,
                   approved_by, approved_at, created_at, updated_at, created_by",
    )
    .bind(payload.analysis_type)
    .bind(id)
    .bind(SqlJson(input_data))
    .bind(payload.requires_human_approval)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "ai_analysis_jobs",
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

async fn find_ai_job(pool: &sqlx::PgPool, id: i64) -> ApiResult<AiJobRow> {
    sqlx::query_as::<_, AiJobRow>(
        "SELECT id, analysis_type, related_vehicle_id, source_table, source_record_id,
                input_data, result_data, confidence_score, job_status, requires_human_approval,
                approved_by, approved_at, created_at, updated_at, created_by
         FROM ai_analysis_jobs
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn validate_analysis_type(analysis_type: &str) -> ApiResult<()> {
    match analysis_type {
        "ocr_verification"
        | "mobile_media_interpretation"
        | "vehicle_media_inspection"
        | "support_media_analysis"
        | "cost_analysis"
        | "maintenance_prediction"
        | "anomaly_detection"
        | "fuel_analysis"
        | "message_generation"
        | "vehicle_risk_summary"
        | "dashboard_insight"
        | "policy_renewal_advice"
        | "damage_claim_assistant"
        | "value_loss_assistant"
        | "import_mapping_assistant"
        | "report_narrative"
        | "support_ticket_triage"
        | "support_sla_risk"
        | "support_root_cause_summary"
        | "support_reply_draft" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid analysis type".to_string())),
    }
}

fn validate_job_status(status: &str) -> ApiResult<()> {
    match status {
        "pending" | "queued" | "running" | "completed" | "failed" | "approved" | "rejected" => {
            Ok(())
        }
        _ => Err(ApiError::BadRequest("invalid ai job status".to_string())),
    }
}

fn required_ai_env(provider: &str) -> Vec<&'static str> {
    match provider {
        "disabled" | "" => vec!["AI_PROVIDER", "AI_MODEL"],
        "local" | "ollama" | "lmstudio" | "openai-compatible" => {
            vec!["AI_PROVIDER", "AI_MODEL", "AI_BASE_URL"]
        }
        "openai" | "gemini" | "claude" | "anthropic" => {
            vec!["AI_PROVIDER", "AI_MODEL", "AI_API_KEY"]
        }
        _ => vec!["AI_PROVIDER", "AI_MODEL", "AI_API_KEY"],
    }
}

fn missing_ai_env(
    provider: &str,
    api_key_configured: bool,
    base_url_configured: bool,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if provider == "disabled" || provider.is_empty() {
        missing.push("AI_PROVIDER");
        missing.push("AI_MODEL");
        return missing;
    }
    if matches!(
        provider,
        "local" | "ollama" | "lmstudio" | "openai-compatible"
    ) && !base_url_configured
    {
        missing.push("AI_BASE_URL");
    }
    if matches!(provider, "openai" | "gemini" | "claude" | "anthropic") && !api_key_configured {
        missing.push("AI_API_KEY");
    }
    missing
}

fn ai_next_step(provider: &str, configured: bool) -> String {
    if configured {
        return "Provider ayarlari tamam. AI worker bu sozlesmedeki pending/queued job kayitlarini isleyebilir.".to_string();
    }
    match provider {
        "disabled" | "" => "AI_PROVIDER secilmeli. Lokal deneme icin AI_PROVIDER=ollama, AI_MODEL=llama3.1 ve AI_BASE_URL=http://127.0.0.1:11434 onerilir.".to_string(),
        "local" | "ollama" | "lmstudio" | "openai-compatible" => "Lokal model icin AI_BASE_URL girilmeli ve model servisi backend tarafindan erisilebilir olmalidir.".to_string(),
        "openai" | "gemini" | "claude" | "anthropic" => "Dis model icin AI_API_KEY girilmeli; hassas veri politikasi ve maskeleme kurallari onaylanmalidir.".to_string(),
        _ => "Provider tipi kontrol edilmeli; dis model ise AI_API_KEY, lokal model ise AI_BASE_URL girilmelidir.".to_string(),
    }
}

fn ai_capabilities() -> Vec<AiCapability> {
    vec![
        AiCapability {
            analysis_type: "ocr_verification",
            title: "Mobil KM fotoğraf okuma",
            target_modules: vec!["km_logs", "file_documents", "vehicles", "mobile"],
            human_approval_required: true,
            description: "Mobil uygulamadan gelen kilometre fotoğrafını okuyup kullanıcının girdiği KM ile karşılaştırır; onaysız doğrulama yapmaz.",
        },
        AiCapability {
            analysis_type: "mobile_media_interpretation",
            title: "Mobil fotoğraf/video yorumlama",
            target_modules: vec!["mobile", "file_documents", "support_tickets", "vehicles"],
            human_approval_required: true,
            description: "Mobil uygulamadan gelen fotoğraf veya kısa videoları bağlamıyla birlikte yorumlayıp arıza, hasar, ekran hatası ve eksik bilgi önerisi üretir.",
        },
        AiCapability {
            analysis_type: "vehicle_media_inspection",
            title: "Araç görsel inceleme",
            target_modules: vec!["vehicles", "file_documents", "damages", "maintenance"],
            human_approval_required: true,
            description: "Araç fotoğraflarından hasar, uyarı lambası, lastik, bakım veya servis ihtiyacına dair ön inceleme notu hazırlar.",
        },
        AiCapability {
            analysis_type: "support_media_analysis",
            title: "Destek görsel analizi",
            target_modules: vec!["support_tickets", "file_documents", "support_ticket_events"],
            human_approval_required: true,
            description: "IT destek talebine eklenen ekran görüntüsü, video, log veya dokümanı analiz ederek kategori, öncelik ve çözüm önerisi çıkarır.",
        },
        AiCapability {
            analysis_type: "vehicle_risk_summary",
            title: "Araç risk özeti",
            target_modules: vec!["vehicles", "km_logs", "maintenances", "policies", "damages", "expenses"],
            human_approval_required: true,
            description: "Araç detayındaki KM, bakım, poliçe, hasar, gider ve operasyon kayıtlarını tek risk özetine çevirir.",
        },
        AiCapability {
            analysis_type: "dashboard_insight",
            title: "Yönetim paneli yorumu",
            target_modules: vec!["dashboard", "audit_logs"],
            human_approval_required: false,
            description: "Dashboard metriklerinden öncelikli aksiyon listesini ve yönetim özetini üretir.",
        },
        AiCapability {
            analysis_type: "policy_renewal_advice",
            title: "Poliçe yenileme asistanı",
            target_modules: vec!["insurance_policies", "insurance_quotes"],
            human_approval_required: true,
            description: "Biten/yaklaşan poliçeleri ve teklifleri karşılaştırıp yenileme önerisi hazırlar.",
        },
        AiCapability {
            analysis_type: "damage_claim_assistant",
            title: "Hasar dosya asistanı",
            target_modules: vec!["damages", "files", "insurance_policies"],
            human_approval_required: true,
            description: "Hasar açıklaması, ekspertiz, fotoğraf ve sigorta dosya durumunu süreç önerisine çevirir.",
        },
        AiCapability {
            analysis_type: "value_loss_assistant",
            title: "Değer kaybı/hak mahrumiyeti asistanı",
            target_modules: vec!["value_loss_claims", "damages", "vehicles"],
            human_approval_required: true,
            description: "Kaza tarihi, tramer, talep, ödeme ve dava notlarından takip önerisi çıkarır.",
        },
        AiCapability {
            analysis_type: "fuel_analysis",
            title: "Yakıt-KM analiz asistanı",
            target_modules: vec!["fuel_entries", "km_logs", "expenses"],
            human_approval_required: false,
            description: "Yakıt limiti, ödenen tutar, gidilen KM ve şüpheli KM kayıtlarından anomali çıkarır.",
        },
        AiCapability {
            analysis_type: "import_mapping_assistant",
            title: "Excel import kolon eşleştirme",
            target_modules: vec!["imports", "vehicles", "policies", "fuel_entries"],
            human_approval_required: true,
            description: "Yüklenen Excel kolonlarını sistem alanlarına eşleştirmek için öneri üretir.",
        },
        AiCapability {
            analysis_type: "message_generation",
            title: "Bildirim metni üretimi",
            target_modules: vec!["notifications", "tasks"],
            human_approval_required: true,
            description: "WhatsApp/e-posta/sistem bildirimi için kontrollü mesaj taslağı üretir.",
        },
        AiCapability {
            analysis_type: "support_ticket_triage",
            title: "IT destek talep sınıflandırma",
            target_modules: vec!["support_tickets", "support_ticket_events", "notifications"],
            human_approval_required: true,
            description: "Destek talebini kategori, öncelik, atama ve ilk aksiyon önerisine çevirir.",
        },
        AiCapability {
            analysis_type: "support_sla_risk",
            title: "IT destek SLA risk analizi",
            target_modules: vec!["support_tickets", "dashboard"],
            human_approval_required: false,
            description: "SLA geciken veya yaklaşan talepleri analiz ederek operasyonel risk ve öncelik üretir.",
        },
        AiCapability {
            analysis_type: "support_root_cause_summary",
            title: "IT kök neden özeti",
            target_modules: vec!["support_tickets", "support_ticket_events"],
            human_approval_required: false,
            description: "Benzer destek taleplerinden tekrar eden sorun, kaynak ve iyileştirme notu çıkarır.",
        },
        AiCapability {
            analysis_type: "support_reply_draft",
            title: "Destek cevap taslağı",
            target_modules: vec!["support_tickets", "notifications"],
            human_approval_required: true,
            description: "WhatsApp veya mobil kullanıcıya gönderilecek çözüm/geri dönüş mesajını taslak olarak üretir.",
        },
        AiCapability {
            analysis_type: "report_narrative",
            title: "Rapor anlatımı",
            target_modules: vec!["reports", "dashboard"],
            human_approval_required: false,
            description: "Yönetim Excel raporundaki sayıları okunabilir kısa özet metne dönüştürür.",
        },
    ]
}

fn prompt_templates() -> Vec<AiPromptTemplate> {
    vec![
        AiPromptTemplate {
            analysis_type: "ocr_verification",
            system_goal: "Mobil uygulamadan gelen kilometre fotografini oku, girilen KM ile karsilastir ve yalnizca insan onayina gidecek kanitli sonuc uret.",
            required_context: vec!["vehicle", "km_log", "file_document", "image_path", "submitted_km", "previous_km"],
            output_contract: vec!["detected_km", "confidence", "evidence", "mismatch_with_submitted_km", "recommended_verification_status", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "mobile_media_interpretation",
            system_goal: "Mobil kullanicidan gelen fotograf/video/dokumani baglamiyla yorumla, talebi netlestir ve eksik bilgi varsa sorulacak sorulari uret.",
            required_context: vec!["source_module", "source_record", "file_document", "mime_type", "note", "reporter", "vehicle"],
            output_contract: vec!["summary", "detected_issue", "severity_suggestion", "missing_information", "recommended_actions", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "vehicle_media_inspection",
            system_goal: "Aracla ilgili yuklenen gorseli veya kisa videoyu incele, servis/hasar/bakim ihtimali icin kanitli on inceleme uret.",
            required_context: vec!["vehicle", "file_document", "maintenance_context", "damage_context", "user_note"],
            output_contract: vec!["summary", "visual_findings", "risk_level", "recommended_actions", "evidence", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "support_media_analysis",
            system_goal: "IT destek talebine eklenen ekran goruntusu, video, log veya dokumani analiz edip uygulanabilir destek aksiyonu oner.",
            required_context: vec!["ticket", "events", "file_document", "mime_type", "reporter", "sla"],
            output_contract: vec!["summary", "category_suggestion", "priority_suggestion", "probable_cause", "first_action", "reply_draft", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "vehicle_risk_summary",
            system_goal: "Arac baglamini incele, riskleri ve oncelikli aksiyonlari kisa ve denetlenebilir bicimde uret.",
            required_context: vec!["vehicle", "latest_km", "maintenance_summary", "policy_summary", "damage_summary", "expense_summary", "operations_summary", "recent_tasks"],
            output_contract: vec!["summary", "risk_level", "evidence", "recommended_actions", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "policy_renewal_advice",
            system_goal: "Police ve teklif verilerini karsilastir, bitis riski ve yenileme onceligini belirt.",
            required_context: vec!["policy_summary", "insurance_quotes", "vehicle"],
            output_contract: vec!["summary", "preferred_quote", "price_notes", "missing_information", "recommended_actions"],
        },
        AiPromptTemplate {
            analysis_type: "damage_claim_assistant",
            system_goal: "Hasar dosyasini surec, eksik evrak ve sigorta takip aksiyonu acisindan ozetle.",
            required_context: vec!["damage_summary", "files", "policy_summary", "vehicle"],
            output_contract: vec!["summary", "claim_status", "missing_documents", "recommended_actions"],
        },
        AiPromptTemplate {
            analysis_type: "value_loss_assistant",
            system_goal: "Deger kaybi ve hak mahrumiyeti kaydini takip edilebilir aksiyonlara cevir.",
            required_context: vec!["value_loss_claims", "damage_summary", "vehicle"],
            output_contract: vec!["summary", "financial_gap", "process_status", "recommended_actions"],
        },
        AiPromptTemplate {
            analysis_type: "fuel_analysis",
            system_goal: "Yakit, KM ve gider verilerinden anomali veya verimlilik uyarisi cikar.",
            required_context: vec!["fuel_entries", "latest_km", "expense_summary"],
            output_contract: vec!["summary", "anomalies", "efficiency_notes", "recommended_actions"],
        },
        AiPromptTemplate {
            analysis_type: "import_mapping_assistant",
            system_goal: "Excel kolonlarini ERP alanlariyla eslestirmek icin guvenli mapping oner.",
            required_context: vec!["raw_headers", "sample_rows", "target_module"],
            output_contract: vec!["column_mapping", "confidence", "unmapped_columns", "warnings"],
        },
        AiPromptTemplate {
            analysis_type: "message_generation",
            system_goal: "Kisa, net, onaya tabi bildirim mesaji taslagi uret.",
            required_context: vec!["event", "vehicle", "recipient_role"],
            output_contract: vec!["message", "channel", "requires_approval"],
        },
        AiPromptTemplate {
            analysis_type: "support_ticket_triage",
            system_goal: "Destek talebini siniflandir, oncelik ve ilk aksiyon oner; status/atama degisikligini otomatik uygulama.",
            required_context: vec!["ticket", "events", "sla", "reporter", "assignment"],
            output_contract: vec!["category_suggestion", "priority_suggestion", "assignment_hint", "first_action", "requires_human_approval"],
        },
        AiPromptTemplate {
            analysis_type: "support_sla_risk",
            system_goal: "SLA hedeflerini ve ticket durumunu incele, gecikme riskini ve operasyon onceligini uret.",
            required_context: vec!["ticket", "events", "sla", "created_at", "first_response_at"],
            output_contract: vec!["sla_risk_level", "missed_target", "recommended_actions", "customer_impact"],
        },
        AiPromptTemplate {
            analysis_type: "support_root_cause_summary",
            system_goal: "Destek talebi ve olay gecmisinden tekrar eden sorun, kok neden ve kalici cozum notu cikar.",
            required_context: vec!["ticket", "events", "resolution_note", "satisfaction"],
            output_contract: vec!["root_cause", "prevention_note", "knowledge_base_summary", "follow_up_actions"],
        },
        AiPromptTemplate {
            analysis_type: "support_reply_draft",
            system_goal: "Kullanicinin anlayacagi netlikte, onaya tabi WhatsApp/mobil destek cevabi taslagi uret.",
            required_context: vec!["ticket", "events", "resolution_note", "source_channel", "reporter"],
            output_contract: vec!["message", "tone", "requires_approval", "missing_information"],
        },
        AiPromptTemplate {
            analysis_type: "report_narrative",
            system_goal: "Yonetim raporu metriklerini patronun okuyabilecegi kisa ozet metne cevir.",
            required_context: vec!["dashboard", "report_metrics"],
            output_contract: vec!["executive_summary", "risks", "recommended_actions"],
        },
    ]
}

fn build_recommendations(signals: RecommendationSignals) -> Vec<AiRecommendation> {
    let mut items = Vec::new();
    push_recommendation(
        &mut items,
        signals.ended_policies,
        "policy_expired",
        "critical",
        "Süresi bitmiş poliçeler için AI yenileme analizi çalıştırılmalı.",
        "Sigortasız operasyon riski oluşmaması için bitmiş poliçeleri teklif ve yenileme akışına alın.",
        "insurance_policies",
        "policy_renewal_advice",
    );
    push_recommendation(
        &mut items,
        signals.expiring_policies,
        "policy_expiring",
        "warning",
        "30 gün içinde bitecek poliçeler için teklif karşılaştırması hazırlanmalı.",
        "AI, mevcut poliçe ve sigorta tekliflerini karşılaştırıp yenileme önceliği çıkarabilir.",
        "insurance_policies",
        "policy_renewal_advice",
    );
    push_recommendation(
        &mut items,
        signals.open_damages,
        "open_damage",
        "warning",
        "Açık hasar dosyaları için süreç özeti hazırlanmalı.",
        "AI, hasar notlarını, dosya numaralarını ve ekspertiz sürecini takip listesine çevirebilir.",
        "damages",
        "damage_claim_assistant",
    );
    push_recommendation(
        &mut items,
        signals.open_value_loss_claims,
        "open_value_loss",
        "warning",
        "Değer kaybı/hak mahrumiyeti dosyaları için takip önerisi üretilebilir.",
        "AI, talep edilen ve gelen tutar farkını, bekleyen dava/not alanlarını özetleyebilir.",
        "value_loss_claims",
        "value_loss_assistant",
    );
    push_recommendation(
        &mut items,
        signals.suspicious_km_logs,
        "suspicious_km",
        "critical",
        "Şüpheli KM kayıtları için anomali analizi çalıştırılmalı.",
        "AI, son KM akışını yakıt ve bakım kayıtlarıyla beraber kontrol edebilir.",
        "km_logs",
        "anomaly_detection",
    );
    push_recommendation(
        &mut items,
        signals.overdue_tasks + signals.critical_tasks,
        "task_pressure",
        "warning",
        "Geciken/kritik görevler için operasyon önceliklendirmesi yapılmalı.",
        "AI, görevleri araç/poliçe/hasar etkisine göre sıralayabilir.",
        "tasks",
        "dashboard_insight",
    );
    push_recommendation(
        &mut items,
        signals.failed_import_jobs,
        "failed_import",
        "info",
        "Hatalı import işlerindeki kolon eşleştirmeleri AI ile incelenebilir.",
        "AI, hatalı satır ve kolon adlarından eşleştirme önerisi çıkarabilir.",
        "imports",
        "import_mapping_assistant",
    );
    push_recommendation(
        &mut items,
        signals.overdue_support_tickets,
        "support_sla_overdue",
        "critical",
        "SLA süresi geçen IT destek talepleri için AI öncelik analizi çalıştırılmalı.",
        "AI, geciken destek taleplerini kullanıcı etkisi ve çözüm hedeflerine göre sıralayabilir.",
        "support_tickets",
        "support_sla_risk",
    );
    push_recommendation(
        &mut items,
        signals.critical_support_tickets,
        "support_critical",
        "critical",
        "Kritik IT destek talepleri için triage önerisi hazırlanmalı.",
        "AI, kategori, atama ve ilk aksiyon önerisi üretip IT ekibinin hızlı karar vermesine destek olabilir.",
        "support_tickets",
        "support_ticket_triage",
    );
    push_recommendation(
        &mut items,
        signals.whatsapp_support_tickets,
        "support_whatsapp_reply",
        "warning",
        "WhatsApp kaynaklı açık destek talepleri için cevap taslakları hazırlanabilir.",
        "AI, kullanıcıya gönderilecek net ve kontrollü dönüş mesajı taslağı oluşturabilir.",
        "support_tickets",
        "support_reply_draft",
    );
    push_recommendation(
        &mut items,
        signals.open_support_tickets,
        "support_root_cause",
        "info",
        "Açık IT destek taleplerinden tekrar eden sorun analizi çıkarılabilir.",
        "AI, destek taleplerini gruplayıp kök neden ve kalıcı iyileştirme notu hazırlayabilir.",
        "support_tickets",
        "support_root_cause_summary",
    );
    items
}

fn push_recommendation(
    items: &mut Vec<AiRecommendation>,
    count: i64,
    recommendation_type: &str,
    severity: &str,
    title: &str,
    detail: &str,
    source_module: &str,
    suggested_analysis_type: &str,
) {
    if count > 0 {
        items.push(AiRecommendation {
            recommendation_type: recommendation_type.to_string(),
            severity: severity.to_string(),
            title: title.to_string(),
            detail: detail.to_string(),
            source_module: source_module.to_string(),
            source_count: count,
            suggested_analysis_type: suggested_analysis_type.to_string(),
        });
    }
}

fn build_task_draft(job: &AiJobRow) -> AiTaskDraft {
    let summary = extract_result_text(job)
        .unwrap_or_else(|| format!("AI analiz sonucu incelenmeli: {}", job.analysis_type));
    AiTaskDraft {
        task_type: format!("ai_{}", job.analysis_type),
        related_vehicle_id: job.related_vehicle_id,
        priority: if job.requires_human_approval {
            "high"
        } else {
            "medium"
        }
        .to_string(),
        description: format!("AI job #{} sonucu: {}", job.id, summary),
        source_ai_job_id: job.id,
    }
}

fn build_notification_draft(job: &AiJobRow) -> AiNotificationDraft {
    let summary = extract_result_text(job)
        .unwrap_or_else(|| format!("{} analiz sonucu hazır.", job.analysis_type));
    AiNotificationDraft {
        notification_type: "ai_result".to_string(),
        related_vehicle_id: job.related_vehicle_id,
        sent_via: "system".to_string(),
        delivery_status: "draft".to_string(),
        message: format!(
            "AI analiz sonucu kontrol bekliyor. Job #{}: {}",
            job.id, summary
        ),
        source_ai_job_id: job.id,
    }
}

fn extract_result_text(job: &AiJobRow) -> Option<String> {
    let result = job.result_data.as_ref()?;
    result
        .get("summary")
        .and_then(Value::as_str)
        .or_else(|| result.get("executive_summary").and_then(Value::as_str))
        .or_else(|| result.get("message").and_then(Value::as_str))
        .map(str::to_string)
}

async fn build_vehicle_context(pool: &sqlx::PgPool, vehicle_id: i64) -> ApiResult<Value> {
    let context = sqlx::query_scalar::<_, SqlJson<Value>>(
        "SELECT jsonb_build_object(
            'vehicle', (
                SELECT to_jsonb(v) - 'deleted_at' - 'created_by' - 'updated_by'
                FROM vehicles v
                WHERE v.id = $1 AND v.deleted_at IS NULL
            ),
            'latest_km', (
                SELECT to_jsonb(kl)
                FROM (
                    SELECT id, km, entry_type::text AS entry_type, verification_status::text AS verification_status, created_at
                    FROM km_logs
                    WHERE vehicle_id = $1 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    LIMIT 1
                ) kl
            ),
            'maintenance_summary', (
                SELECT jsonb_build_object(
                    'total', count(*),
                    'planned', count(*) FILTER (WHERE maintenance_status = 'planned'::maintenance_status),
                    'scheduled', count(*) FILTER (WHERE maintenance_status = 'scheduled'::maintenance_status),
                    'completed', count(*) FILTER (WHERE maintenance_status = 'completed'::maintenance_status),
                    'total_cost', sum(total_cost)
                )
                FROM maintenances
                WHERE vehicle_id = $1 AND deleted_at IS NULL
            ),
            'policy_summary', (
                SELECT jsonb_build_object(
                    'total', count(*),
                    'active', count(*) FILTER (WHERE renewal_status = 'active'::renewal_status),
                    'ending_in_30_days', count(*) FILTER (WHERE end_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '30 days'),
                    'latest_end_date', max(end_date)
                )
                FROM insurance_policies
                WHERE vehicle_id = $1 AND deleted_at IS NULL
            ),
            'damage_summary', (
                SELECT jsonb_build_object(
                    'total', count(*),
                    'open', count(*) FILTER (WHERE damage_status IN ('open', 'expertise', 'insurance')),
                    'estimated_cost', sum(estimated_cost),
                    'actual_cost', sum(actual_cost)
                )
                FROM damages
                WHERE vehicle_id = $1 AND deleted_at IS NULL
            ),
            'expense_summary', (
                SELECT jsonb_build_object(
                    'total', count(*),
                    'total_amount', sum(amount),
                    'pending', count(*) FILTER (WHERE payment_status = 'pending'),
                    'overdue', count(*) FILTER (WHERE payment_status = 'overdue')
                )
                FROM expenses
                WHERE vehicle_id = $1 AND deleted_at IS NULL
            ),
            'operations_summary', jsonb_build_object(
                'inspections', (SELECT count(*) FROM vehicle_inspections WHERE vehicle_id = $1 AND deleted_at IS NULL),
                'value_loss_claims', (SELECT count(*) FROM value_loss_claims WHERE vehicle_id = $1 AND deleted_at IS NULL),
                'fuel_entries', (SELECT count(*) FROM fuel_entries WHERE vehicle_id = $1 AND deleted_at IS NULL),
                'washes', (SELECT count(*) FROM vehicle_washes WHERE vehicle_id = $1 AND deleted_at IS NULL),
                'quotes', (SELECT count(*) FROM insurance_quotes WHERE vehicle_id = $1 AND deleted_at IS NULL)
            ),
            'recent_tasks', (
                SELECT COALESCE(jsonb_agg(to_jsonb(t)), '[]'::jsonb)
                FROM (
                    SELECT id, task_type, priority::text AS priority, due_date, task_status, description, created_at
                    FROM tasks
                    WHERE related_vehicle_id = $1 AND deleted_at IS NULL
                    ORDER BY created_at DESC
                    LIMIT 10
                ) t
            )
        )",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let value = context.0;
    if value.get("vehicle").map(Value::is_null).unwrap_or(true) {
        return Err(ApiError::NotFound);
    }
    Ok(value)
}

async fn build_support_ticket_context(pool: &sqlx::PgPool, ticket_id: i64) -> ApiResult<Value> {
    let context = sqlx::query_scalar::<_, SqlJson<Value>>(
        "SELECT jsonb_build_object(
            'ticket', (
                SELECT jsonb_build_object(
                    'id', t.id,
                    'ticket_no', t.ticket_no,
                    'title', t.title,
                    'description', t.description,
                    'category', t.category,
                    'priority', t.priority::text,
                    'ticket_status', t.ticket_status,
                    'source_channel', t.source_channel,
                    'reporter_name', t.reporter_name,
                    'reporter_phone', t.reporter_phone,
                    'assigned_user_id', t.assigned_user_id,
                    'assigned_department_id', t.assigned_department_id,
                    'resolution_note', t.resolution_note,
                    'created_at', t.created_at,
                    'updated_at', t.updated_at
                )
                FROM support_tickets t
                WHERE t.id = $1 AND t.deleted_at IS NULL
            ),
            'sla', (
                SELECT jsonb_build_object(
                    'sla_status', support_sla_status(t.ticket_status, t.first_response_at, t.sla_response_due_at, t.sla_resolution_due_at),
                    'response_due_at', t.sla_response_due_at,
                    'resolution_due_at', t.sla_resolution_due_at,
                    'first_response_at', t.first_response_at,
                    'resolved_at', t.resolved_at,
                    'closed_at', t.closed_at,
                    'escalation_level', t.escalation_level
                )
                FROM support_tickets t
                WHERE t.id = $1 AND t.deleted_at IS NULL
            ),
            'reporter', (
                SELECT jsonb_build_object(
                    'reporter_user_id', t.reporter_user_id,
                    'reporter_name', t.reporter_name,
                    'reporter_phone', t.reporter_phone
                )
                FROM support_tickets t
                WHERE t.id = $1 AND t.deleted_at IS NULL
            ),
            'assignment', (
                SELECT jsonb_build_object(
                    'assigned_user', u.full_name,
                    'assigned_department', d.name
                )
                FROM support_tickets t
                LEFT JOIN users u ON u.id = t.assigned_user_id
                LEFT JOIN departments d ON d.id = t.assigned_department_id
                WHERE t.id = $1 AND t.deleted_at IS NULL
            ),
            'satisfaction', (
                SELECT jsonb_build_object(
                    'score', t.satisfaction_score,
                    'note', t.satisfaction_note
                )
                FROM support_tickets t
                WHERE t.id = $1 AND t.deleted_at IS NULL
            ),
            'events', (
                SELECT COALESCE(jsonb_agg(to_jsonb(e) ORDER BY e.created_at DESC), '[]'::jsonb)
                FROM (
                    SELECT event_type, note, old_status, new_status, created_at
                    FROM support_ticket_events
                    WHERE ticket_id = $1
                    ORDER BY created_at DESC
                    LIMIT 25
                ) e
            ),
            'safety_contract', jsonb_build_object(
                'no_automatic_status_change', true,
                'no_automatic_customer_message', true,
                'human_approval_required_for_reply', true
            )
        )",
    )
    .bind(ticket_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    let value = context.0;
    if value.get("ticket").map(Value::is_null).unwrap_or(true) {
        return Err(ApiError::NotFound);
    }
    Ok(value)
}
