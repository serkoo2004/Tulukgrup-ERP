use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    support, AppState,
};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{types::Json as SqlJson, FromRow};
use std::{env, time::Duration};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_notifications).post(create_notification))
        .route("/mine", get(list_my_notifications))
        .route("/provider-status", get(get_provider_status))
        .route(
            "/whatsapp/webhook",
            get(verify_whatsapp_webhook).post(receive_whatsapp_webhook),
        )
        .route("/:id/dispatch", post(dispatch_notification))
        .route("/:id/read", post(mark_notification_read))
        .route("/:id", get(get_notification).patch(update_delivery_status))
}

#[derive(Debug, Deserialize)]
struct NotificationQuery {
    receiver_user_id: Option<i64>,
    related_vehicle_id: Option<i64>,
    sent_via: Option<String>,
    delivery_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct NotificationCreate {
    notification_type: String,
    receiver_user_id: Option<i64>,
    related_vehicle_id: Option<i64>,
    message: String,
    sent_via: String,
    delivery_status: Option<String>,
    recipient_phone: Option<String>,
    provider_payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct DeliveryStatusUpdate {
    delivery_status: String,
    external_message_id: Option<String>,
    delivery_error: Option<String>,
    provider_payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct WhatsAppWebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct NotificationRow {
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
struct ProviderStatusResponse {
    whatsapp: WhatsAppProviderStatus,
}

#[derive(Debug, Serialize)]
struct WhatsAppProviderStatus {
    enabled: bool,
    configured: bool,
    api_url: String,
    phone_number_id_configured: bool,
    access_token_configured: bool,
    webhook_verify_token_configured: bool,
    app_secret_configured: bool,
    template_language: String,
    required_env: Vec<&'static str>,
    missing_env: Vec<&'static str>,
    outbound_mode: &'static str,
    webhook_path: &'static str,
    next_step: String,
}

async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NotificationQuery>,
) -> ApiResult<Json<Vec<NotificationRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = query_notifications(&state.pool, &query).await?;
    Ok(Json(rows))
}

async fn list_my_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(mut query): Query<NotificationQuery>,
) -> ApiResult<Json<Vec<NotificationRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    query.receiver_user_id = Some(current.id);
    Ok(Json(query_notifications(&state.pool, &query).await?))
}

async fn create_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NotificationCreate>,
) -> ApiResult<Json<NotificationRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_channel(&payload.sent_via)?;

    let delivery_status = payload
        .delivery_status
        .unwrap_or_else(|| "pending".to_string());
    validate_delivery_status(&delivery_status)?;

    let row = sqlx::query_as::<_, NotificationRow>(
        "INSERT INTO notifications
         (notification_type, receiver_user_id, related_vehicle_id, message,
          sent_via, delivery_status, recipient_phone, provider_payload, created_by)
         VALUES ($1, $2, $3, $4, $5::notification_channel, $6, $7, $8, $9)
         RETURNING id, notification_type, receiver_user_id, related_vehicle_id, message,
                   sent_via::text AS sent_via, delivery_status, recipient_phone,
                   external_message_id, provider_payload, delivery_error, sent_at, read_at,
                   created_at, updated_at, created_by",
    )
    .bind(payload.notification_type)
    .bind(payload.receiver_user_id)
    .bind(payload.related_vehicle_id)
    .bind(payload.message)
    .bind(payload.sent_via)
    .bind(delivery_status)
    .bind(payload.recipient_phone)
    .bind(payload.provider_payload)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "notifications",
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

async fn get_provider_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ProviderStatusResponse>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(ProviderStatusResponse {
        whatsapp: whatsapp_provider_status(),
    }))
}

async fn get_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<NotificationRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let row = find_notification(&state.pool, id).await?;
    if row.receiver_user_id == Some(current.id)
        || matches!(current.role.as_str(), "admin" | "manager" | "operation")
    {
        Ok(Json(row))
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn verify_whatsapp_webhook(
    Query(query): Query<WhatsAppWebhookVerifyQuery>,
) -> ApiResult<String> {
    let expected = env::var("WHATSAPP_WEBHOOK_VERIFY_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "change-me".to_string());
    let is_valid = query.mode.as_deref() == Some("subscribe")
        && query.verify_token.as_deref() == Some(expected.as_str())
        && query
            .challenge
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
    if is_valid {
        Ok(query.challenge.unwrap_or_default())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn receive_whatsapp_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Value>> {
    verify_whatsapp_signature(&headers, &body)?;
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::BadRequest("invalid whatsapp webhook json".to_string()))?;
    sqlx::query(
        "INSERT INTO system_error_logs (service_name, severity, message, context)
         VALUES ('whatsapp_webhook', 'info', 'WhatsApp webhook payload received', $1)",
    )
    .bind(SqlJson(payload.clone()))
    .execute(&state.pool)
    .await?;

    let ticket_inputs = extract_whatsapp_ticket_inputs(&payload);
    let mut ticket_ids = Vec::new();
    for input in ticket_inputs {
        let ticket = support::create_ticket_from_whatsapp_message(&state.pool, input).await?;
        ticket_ids.push(ticket.id);
    }

    Ok(Json(
        json!({ "ok": true, "support_ticket_ids": ticket_ids }),
    ))
}

async fn update_delivery_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<DeliveryStatusUpdate>,
) -> ApiResult<Json<NotificationRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_delivery_status(&payload.delivery_status)?;

    let old = find_notification(&state.pool, id).await?;
    let row = sqlx::query_as::<_, NotificationRow>(
        "UPDATE notifications
         SET delivery_status = $2,
             sent_at = CASE WHEN $2 = 'sent' THEN COALESCE(sent_at, now()) ELSE sent_at END,
             external_message_id = COALESCE($3, external_message_id),
             delivery_error = $4,
             provider_payload = COALESCE($5, provider_payload),
             updated_by = $6,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, notification_type, receiver_user_id, related_vehicle_id, message,
                   sent_via::text AS sent_via, delivery_status, recipient_phone,
                   external_message_id, provider_payload, delivery_error, sent_at, read_at,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.delivery_status)
    .bind(payload.external_message_id)
    .bind(payload.delivery_error)
    .bind(payload.provider_payload)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "notifications",
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

async fn dispatch_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<NotificationRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let old = find_notification(&state.pool, id).await?;
    if old.sent_via != "whatsapp" {
        return Err(ApiError::BadRequest(
            "only whatsapp notifications can be dispatched".to_string(),
        ));
    }

    let recipient_phone = old
        .recipient_phone
        .clone()
        .filter(|phone| !phone.trim().is_empty())
        .ok_or_else(|| {
            ApiError::BadRequest("recipient_phone is required for whatsapp dispatch".to_string())
        })?;

    let (payload, delivery_status, delivery_error, external_message_id) =
        dispatch_whatsapp_message(&old.notification_type, &recipient_phone, &old.message).await;

    let row = sqlx::query_as::<_, NotificationRow>(
        "UPDATE notifications
         SET delivery_status = $2,
             provider_payload = $3,
             delivery_error = $4,
             external_message_id = COALESCE($6, external_message_id),
             sent_at = CASE WHEN $2 = 'sent' THEN COALESCE(sent_at, now()) ELSE sent_at END,
             updated_by = $5,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, notification_type, receiver_user_id, related_vehicle_id, message,
                   sent_via::text AS sent_via, delivery_status, recipient_phone,
                   external_message_id, provider_payload, delivery_error, sent_at, read_at,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(delivery_status)
    .bind(payload)
    .bind(delivery_error)
    .bind(current.id)
    .bind(external_message_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "notifications",
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

pub async fn dispatch_whatsapp_message(
    notification_type: &str,
    recipient_phone: &str,
    message: &str,
) -> (Value, String, Option<String>, Option<String>) {
    let provider = whatsapp_provider_status();
    let request_payload = build_whatsapp_text_payload(recipient_phone, message);
    let base_payload = json!({
        "provider": "whatsapp_business_api",
        "api_url": provider.api_url,
        "phone_number_id_configured": provider.phone_number_id_configured,
        "template_language": provider.template_language,
        "to": recipient_phone,
        "notification_type": notification_type,
        "message": message,
        "request": request_payload
    });

    if !provider.configured {
        return (
            base_payload,
            "failed".to_string(),
            Some("whatsapp provider is not configured; set WHATSAPP_PHONE_NUMBER_ID and WHATSAPP_ACCESS_TOKEN".to_string()),
            None,
        );
    }

    let phone_number_id = match env::var("WHATSAPP_PHONE_NUMBER_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        Some(value) => value,
        None => {
            return (
                base_payload,
                "failed".to_string(),
                Some("WHATSAPP_PHONE_NUMBER_ID is missing".to_string()),
                None,
            )
        }
    };
    let access_token = match env::var("WHATSAPP_ACCESS_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        Some(value) => value,
        None => {
            return (
                base_payload,
                "failed".to_string(),
                Some("WHATSAPP_ACCESS_TOKEN is missing".to_string()),
                None,
            )
        }
    };

    let timeout_seconds = env::var("WHATSAPP_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(20)
        .clamp(5, 120);
    let url = format!(
        "{}/{}/messages",
        provider.api_url.trim_end_matches('/'),
        phone_number_id
    );
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return (
                base_payload,
                "failed".to_string(),
                Some(format!("whatsapp client build failed: {error}")),
                None,
            )
        }
    };

    let response = client
        .post(&url)
        .bearer_auth(access_token)
        .json(&request_payload)
        .send()
        .await;
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            return (
                json!({ "request": base_payload, "endpoint": url }),
                "failed".to_string(),
                Some(format!("whatsapp send failed: {error}")),
                None,
            )
        }
    };

    let status = response.status();
    let response_body = response.json::<Value>().await.unwrap_or_else(|error| {
        json!({
            "parse_error": error.to_string()
        })
    });
    let provider_payload = json!({
        "request": base_payload,
        "endpoint": url,
        "http_status": status.as_u16(),
        "response": response_body
    });

    if status.is_success() {
        let external_message_id = extract_whatsapp_message_id(&provider_payload);
        (
            provider_payload,
            "sent".to_string(),
            None,
            external_message_id,
        )
    } else {
        let delivery_error = extract_whatsapp_error(&provider_payload);
        (
            provider_payload,
            "failed".to_string(),
            delivery_error,
            None,
        )
    }
}

fn build_whatsapp_text_payload(recipient_phone: &str, message: &str) -> Value {
    json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": recipient_phone,
        "type": "text",
        "text": {
            "preview_url": false,
            "body": message
        }
    })
}

fn extract_whatsapp_message_id(payload: &Value) -> Option<String> {
    payload
        .get("response")
        .and_then(|response| response.get("messages"))
        .and_then(Value::as_array)
        .and_then(|messages| messages.first())
        .and_then(|message| message.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn extract_whatsapp_error(payload: &Value) -> Option<String> {
    payload
        .get("response")
        .and_then(|response| response.get("error"))
        .and_then(|error| {
            error
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| error.get("error_user_msg").and_then(Value::as_str))
        })
        .map(str::to_string)
        .or_else(|| Some("whatsapp provider returned an error".to_string()))
}

async fn mark_notification_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<NotificationRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let old = find_notification(&state.pool, id).await?;
    if old.receiver_user_id != Some(current.id)
        && !matches!(current.role.as_str(), "admin" | "manager" | "operation")
    {
        return Err(ApiError::Forbidden);
    }

    let row = sqlx::query_as::<_, NotificationRow>(
        "UPDATE notifications
         SET read_at = COALESCE(read_at, now()),
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, notification_type, receiver_user_id, related_vehicle_id, message,
                   sent_via::text AS sent_via, delivery_status, recipient_phone,
                   external_message_id, provider_payload, delivery_error, sent_at, read_at,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "notifications",
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

async fn query_notifications(
    pool: &sqlx::PgPool,
    query: &NotificationQuery,
) -> ApiResult<Vec<NotificationRow>> {
    Ok(sqlx::query_as::<_, NotificationRow>(
        "SELECT id, notification_type, receiver_user_id, related_vehicle_id, message,
                sent_via::text AS sent_via, delivery_status, recipient_phone,
                external_message_id, provider_payload, delivery_error, sent_at, read_at,
                created_at, updated_at, created_by
         FROM notifications
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR receiver_user_id = $1)
           AND ($2::bigint IS NULL OR related_vehicle_id = $2)
           AND ($3::text IS NULL OR sent_via::text = $3)
           AND ($4::text IS NULL OR delivery_status = $4)
         ORDER BY created_at DESC
         LIMIT $5",
    )
    .bind(query.receiver_user_id)
    .bind(query.related_vehicle_id)
    .bind(query.sent_via.clone())
    .bind(query.delivery_status.clone())
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(pool)
    .await?)
}

async fn find_notification(pool: &sqlx::PgPool, id: i64) -> ApiResult<NotificationRow> {
    sqlx::query_as::<_, NotificationRow>(
        "SELECT id, notification_type, receiver_user_id, related_vehicle_id, message,
                sent_via::text AS sent_via, delivery_status, recipient_phone,
                external_message_id, provider_payload, delivery_error, sent_at, read_at,
                created_at, updated_at, created_by
         FROM notifications
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn whatsapp_provider_status() -> WhatsAppProviderStatus {
    let api_url = env::var("WHATSAPP_BUSINESS_API_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://graph.facebook.com/v20.0".to_string());
    let phone_number_id_configured = env_is_configured("WHATSAPP_PHONE_NUMBER_ID");
    let access_token_configured = env_is_configured("WHATSAPP_ACCESS_TOKEN");
    let webhook_verify_token_configured = env_is_configured("WHATSAPP_WEBHOOK_VERIFY_TOKEN");
    let app_secret_configured = env_is_configured("WHATSAPP_APP_SECRET");
    let enabled = env::var("WHATSAPP_ENABLED")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);
    let template_language = env::var("WHATSAPP_TEMPLATE_LANGUAGE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "tr".to_string());

    let missing_env =
        whatsapp_missing_env(enabled, phone_number_id_configured, access_token_configured);
    WhatsAppProviderStatus {
        enabled,
        configured: enabled && missing_env.is_empty(),
        api_url,
        phone_number_id_configured,
        access_token_configured,
        webhook_verify_token_configured,
        app_secret_configured,
        template_language,
        required_env: vec![
            "WHATSAPP_ENABLED",
            "WHATSAPP_BUSINESS_API_URL",
            "WHATSAPP_PHONE_NUMBER_ID",
            "WHATSAPP_ACCESS_TOKEN",
            "WHATSAPP_TEMPLATE_LANGUAGE",
            "WHATSAPP_APP_SECRET",
        ],
        missing_env,
        outbound_mode: "direct_graph_api_dispatch",
        webhook_path: "/api/v1/notifications/whatsapp/webhook",
        next_step: whatsapp_next_step(enabled, phone_number_id_configured, access_token_configured),
    }
}

fn whatsapp_missing_env(
    enabled: bool,
    phone_number_id_configured: bool,
    access_token_configured: bool,
) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if !enabled {
        missing.push("WHATSAPP_ENABLED");
    }
    if !phone_number_id_configured {
        missing.push("WHATSAPP_PHONE_NUMBER_ID");
    }
    if !access_token_configured {
        missing.push("WHATSAPP_ACCESS_TOKEN");
    }
    missing
}

fn whatsapp_next_step(
    enabled: bool,
    phone_number_id_configured: bool,
    access_token_configured: bool,
) -> String {
    if enabled && phone_number_id_configured && access_token_configured {
        return "WhatsApp ayarlari tamam. Dispatch endpointi Graph API'ye dogrudan mesaj gonderebilir.".to_string();
    }
    if !enabled {
        return "WHATSAPP_ENABLED=true yapilmali ve Meta WhatsApp Business API bilgileri girilmeli.".to_string();
    }
    "Meta panelinden phone number id ve access token alinip WHATSAPP_PHONE_NUMBER_ID/WHATSAPP_ACCESS_TOKEN olarak girilmeli.".to_string()
}

fn env_is_configured(key: &str) -> bool {
    env::var(key)
        .map(|value| !value.trim().is_empty() && !value.trim().eq_ignore_ascii_case("change-me"))
        .unwrap_or(false)
}

fn extract_whatsapp_ticket_inputs(payload: &Value) -> Vec<support::WhatsAppTicketInput> {
    let mut tickets = Vec::new();
    let Some(entries) = payload.get("entry").and_then(Value::as_array) else {
        return tickets;
    };
    for entry in entries {
        let Some(changes) = entry.get("changes").and_then(Value::as_array) else {
            continue;
        };
        for change in changes {
            let Some(value) = change.get("value") else {
                continue;
            };
            let contact_name = value
                .get("contacts")
                .and_then(Value::as_array)
                .and_then(|contacts| contacts.first())
                .and_then(|contact| contact.get("profile"))
                .and_then(|profile| profile.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let Some(messages) = value.get("messages").and_then(Value::as_array) else {
                continue;
            };
            for message in messages {
                let text = message
                    .get("text")
                    .and_then(|text| text.get("body"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|body| !body.is_empty());
                let Some(text) = text else {
                    continue;
                };
                let reporter_phone = message
                    .get("from")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let external_message_id = message
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                tickets.push(support::WhatsAppTicketInput {
                    reporter_phone,
                    reporter_name: contact_name.clone(),
                    message: text.to_string(),
                    external_message_id,
                });
            }
        }
    }
    tickets
}

fn verify_whatsapp_signature(headers: &HeaderMap, body: &[u8]) -> ApiResult<()> {
    let Some(secret) = env::var("WHATSAPP_APP_SECRET").ok().filter(|value| {
        !value.trim().is_empty() && !value.trim().eq_ignore_ascii_case("change-me")
    }) else {
        return Ok(());
    };
    let provided = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("sha256="))
        .ok_or_else(|| ApiError::Forbidden)?;
    let expected = hmac_sha256_hex(secret.as_bytes(), body);
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

fn hmac_sha256_hex(key: &[u8], message: &[u8]) -> String {
    const BLOCK_SIZE: usize = 64;
    let mut normalized_key = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let digest = Sha256::digest(key);
        normalized_key[..digest.len()].copy_from_slice(&digest);
    } else {
        normalized_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; BLOCK_SIZE];
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for index in 0..BLOCK_SIZE {
        ipad[index] ^= normalized_key[index];
        opad[index] ^= normalized_key[index];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    hex::encode(outer.finalize())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

fn validate_channel(channel: &str) -> ApiResult<()> {
    match channel {
        "whatsapp" | "system" | "email" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid notification channel".to_string(),
        )),
    }
}

fn validate_delivery_status(status: &str) -> ApiResult<()> {
    match status {
        "pending" | "queued" | "sent" | "failed" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid delivery status".to_string())),
    }
}
