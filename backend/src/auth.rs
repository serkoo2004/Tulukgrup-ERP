use crate::{
    config::Config,
    error::{ApiError, ApiResult},
    AppState,
};
use axum::{
    extract::State,
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/change-password", post(change_password))
        .route("/me", get(me))
}

#[derive(Debug, FromRow, Serialize)]
pub struct AuthUser {
    pub id: i64,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub company_id: Option<i64>,
    pub department_id: Option<i64>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_active: bool,
}

#[derive(Debug, FromRow, Serialize)]
pub struct MobilePermissionClaim {
    pub permission_key: String,
    pub company_id: Option<i64>,
    pub department_id: Option<i64>,
    pub can_view: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_approve: bool,
}

#[derive(Debug, Serialize)]
struct CurrentUserResponse {
    id: i64,
    email: String,
    full_name: String,
    role: String,
    company_id: Option<i64>,
    department_id: Option<i64>,
    is_active: bool,
    mobile_permissions: Vec<MobilePermissionClaim>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub token_type: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
struct MessageResponse {
    message: &'static str,
}

pub async fn seed_bootstrap_admin(pool: &PgPool, config: &Config) -> ApiResult<()> {
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(&config.bootstrap_admin_email)
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        let password_hash = hash(&config.bootstrap_admin_password, DEFAULT_COST)
            .map_err(|err| ApiError::Internal(err.into()))?;
        sqlx::query(
            "INSERT INTO users (email, full_name, password_hash, role, is_active)
             VALUES ($1, 'Sistem Admin', $2, 'admin', true)",
        )
        .bind(&config.bootstrap_admin_email)
        .bind(password_hash)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<Json<TokenResponse>> {
    let user = find_user_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    let password_ok = verify(payload.password, &user.password_hash)
        .map_err(|err| ApiError::Internal(err.into()))?;
    if !password_ok || !user.is_active {
        return Err(ApiError::Unauthorized);
    }

    let tokens = create_token_pair(&state, &user).await?;
    crate::audit::write_audit(
        &state.pool,
        "users",
        Some(user.id),
        "login",
        None,
        serde_json::json!({ "email": user.email }),
        Some(user.id),
        &headers,
    )
    .await?;
    Ok(Json(tokens))
}

async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> ApiResult<Json<TokenResponse>> {
    let claims = decode_token(&payload.refresh_token, &state.config.jwt_refresh_secret)?;
    if claims.token_type != "refresh" {
        return Err(ApiError::Unauthorized);
    }

    let token_hash = hash_token(&payload.refresh_token);
    let user_id: i64 = claims.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let valid: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM refresh_tokens
         WHERE user_id = $1 AND token_hash = $2 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(user_id)
    .bind(token_hash)
    .fetch_optional(&state.pool)
    .await?;

    if valid.is_none() {
        return Err(ApiError::Unauthorized);
    }

    let user = find_user_by_id(&state.pool, user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    Ok(Json(create_token_pair(&state, &user).await?))
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LogoutRequest>,
) -> ApiResult<Json<MessageResponse>> {
    let claims = decode_token(&payload.refresh_token, &state.config.jwt_refresh_secret)?;
    if claims.token_type != "refresh" {
        return Err(ApiError::Unauthorized);
    }

    let user_id: i64 = claims.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let token_hash = hash_token(&payload.refresh_token);
    sqlx::query(
        "UPDATE refresh_tokens
         SET revoked_at = now()
         WHERE user_id = $1 AND token_hash = $2 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .bind(token_hash)
    .execute(&state.pool)
    .await?;

    crate::audit::write_audit(
        &state.pool,
        "users",
        Some(user_id),
        "logout",
        None,
        serde_json::json!({ "refresh_token_revoked": true }),
        Some(user_id),
        &headers,
    )
    .await?;

    Ok(Json(MessageResponse {
        message: "logged out",
    }))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> ApiResult<Json<MessageResponse>> {
    if payload.new_password.len() < 8 {
        return Err(ApiError::BadRequest(
            "new password must be at least 8 characters".to_string(),
        ));
    }
    let current = current_user(&state, &headers).await?;
    let password_ok = verify(payload.current_password, &current.password_hash)
        .map_err(|err| ApiError::Internal(err.into()))?;
    if !password_ok {
        return Err(ApiError::Unauthorized);
    }

    let password_hash =
        hash(payload.new_password, DEFAULT_COST).map_err(|err| ApiError::Internal(err.into()))?;
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE users
         SET password_hash = $2,
             updated_by = $1,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(current.id)
    .bind(password_hash)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(current.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    crate::audit::write_audit(
        &state.pool,
        "users",
        Some(current.id),
        "update",
        None,
        serde_json::json!({ "password_changed": true, "refresh_tokens_revoked": true }),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(MessageResponse {
        message: "password changed",
    }))
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<CurrentUserResponse>> {
    let current = current_user(&state, &headers).await?;
    let mobile_permissions = mobile_permissions_for_user(&state.pool, &current).await?;
    Ok(Json(CurrentUserResponse {
        id: current.id,
        email: current.email,
        full_name: current.full_name,
        role: current.role,
        company_id: current.company_id,
        department_id: current.department_id,
        is_active: current.is_active,
        mobile_permissions,
    }))
}

pub async fn current_user(state: &AppState, headers: &HeaderMap) -> ApiResult<AuthUser> {
    let token = bearer_token(headers)?;
    let claims = decode_token(token, &state.config.jwt_access_secret)?;
    if claims.token_type != "access" {
        return Err(ApiError::Unauthorized);
    }
    let user_id: i64 = claims.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    find_user_by_id(&state.pool, user_id)
        .await?
        .filter(|user| user.is_active)
        .ok_or(ApiError::Unauthorized)
}

pub fn require_roles(user: &AuthUser, roles: &[&str]) -> ApiResult<()> {
    if roles.iter().any(|role| *role == user.role) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

pub async fn has_mobile_permission(
    pool: &PgPool,
    user: &AuthUser,
    permission_key: &str,
    action: &str,
) -> ApiResult<bool> {
    let column_value = sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(bool_or(
             CASE $3
               WHEN 'view' THEN can_view
               WHEN 'create' THEN can_create
               WHEN 'update' THEN can_update
               WHEN 'approve' THEN can_approve
               ELSE false
             END
         ), false)
         FROM mobile_user_permissions
         WHERE user_id = $1
           AND permission_key = $2
           AND is_active = true
           AND deleted_at IS NULL
           AND (company_id IS NULL OR company_id = $4)
           AND (department_id IS NULL OR department_id = $5)",
    )
    .bind(user.id)
    .bind(permission_key)
    .bind(action)
    .bind(user.company_id)
    .bind(user.department_id)
    .fetch_one(pool)
    .await?;

    Ok(column_value)
}

pub async fn require_roles_or_mobile_permission(
    pool: &PgPool,
    user: &AuthUser,
    roles: &[&str],
    permission_key: &str,
    action: &str,
) -> ApiResult<()> {
    if roles.iter().any(|role| *role == user.role)
        || has_mobile_permission(pool, user, permission_key, action).await?
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn mobile_permissions_for_user(
    pool: &PgPool,
    user: &AuthUser,
) -> ApiResult<Vec<MobilePermissionClaim>> {
    let rows = sqlx::query_as::<_, MobilePermissionClaim>(
        "SELECT permission_key, company_id, department_id,
                can_view, can_create, can_update, can_approve
         FROM mobile_user_permissions
         WHERE user_id = $1
           AND is_active = true
           AND deleted_at IS NULL
           AND (company_id IS NULL OR company_id = $2)
           AND (department_id IS NULL OR department_id = $3)
         ORDER BY permission_key, company_id NULLS FIRST, department_id NULLS FIRST",
    )
    .bind(user.id)
    .bind(user.company_id)
    .bind(user.department_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    value.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)
}

async fn create_token_pair(state: &AppState, user: &AuthUser) -> ApiResult<TokenResponse> {
    let access_token = create_token(
        user,
        "access",
        state.config.access_token_expire_minutes,
        &state.config.jwt_access_secret,
    )?;
    let refresh_token = create_token(
        user,
        "refresh",
        state.config.refresh_token_expire_days * 24 * 60,
        &state.config.jwt_refresh_secret,
    )?;
    let token_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + Duration::days(state.config.refresh_token_expire_days);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
         VALUES ($1, $2, $3)",
    )
    .bind(user.id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "bearer",
    })
}

fn create_token(
    user: &AuthUser,
    token_type: &str,
    expire_minutes: i64,
    secret: &str,
) -> ApiResult<String> {
    let exp = (Utc::now() + Duration::minutes(expire_minutes)).timestamp() as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        role: user.role.clone(),
        token_type: token_type.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| ApiError::Internal(err.into()))
}

fn decode_token(token: &str, secret: &str) -> ApiResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::Unauthorized)
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

async fn find_user_by_email(pool: &PgPool, email: &str) -> ApiResult<Option<AuthUser>> {
    Ok(sqlx::query_as::<_, AuthUser>(
        "SELECT id, email, full_name, role::text AS role, company_id, department_id,
                password_hash, is_active
         FROM users WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?)
}

async fn find_user_by_id(pool: &PgPool, id: i64) -> ApiResult<Option<AuthUser>> {
    Ok(sqlx::query_as::<_, AuthUser>(
        "SELECT id, email, full_name, role::text AS role, company_id, department_id,
                password_hash, is_active
         FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}
