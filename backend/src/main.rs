#![recursion_limit = "512"]

mod ai;
mod api_docs;
mod assignments;
mod audit;
mod auth;
mod config;
mod damages;
mod dashboard;
mod error;
mod expenses;
mod files;
mod imports;
mod insurance;
mod inventory;
mod maintenance;
mod notifications;
mod operations;
mod organization;
mod rate_limit;
mod reports;
mod search;
mod settings;
mod support;
mod system_logs;
mod tasks;
mod tracking;
mod users;
mod vehicles;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::HeaderValue,
    middleware,
    routing::get,
    Json, Router,
};
use config::Config;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = Arc::new(Config::from_env()?);
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("../database/migrations").run(&pool).await?;
    auth::seed_bootstrap_admin(&pool, &config).await?;

    let state = AppState {
        pool,
        rate_limiter: Arc::new(rate_limit::RateLimiter::new()),
        config,
    };
    let cors = build_cors_layer(&state.config);
    let app = Router::new()
        .route("/health", get(health))
        .route("/health/details", get(health_details))
        .route("/docs", get(api_docs::swagger_ui))
        .route("/openapi.json", get(api_docs::openapi_json))
        .nest("/api/v1/ai", ai::router())
        .nest("/api/v1/assignments", assignments::router())
        .nest("/api/v1/auth", auth::router())
        .nest("/api/v1/damages", damages::router())
        .nest("/api/v1/dashboard", dashboard::router())
        .nest("/api/v1/expenses", expenses::router())
        .nest("/api/v1/files", files::router())
        .nest("/api/v1/imports", imports::router())
        .nest("/api/v1/inventory", inventory::router())
        .nest("/api/v1/insurance-policies", insurance::router())
        .nest("/api/v1/maintenances", maintenance::router())
        .nest("/api/v1/notifications", notifications::router())
        .nest("/api/v1/operations", operations::router())
        .nest("/api/v1/organization", organization::router())
        .nest("/api/v1/reports", reports::router())
        .nest("/api/v1/search", search::router())
        .nest("/api/v1/settings", settings::router())
        .nest("/api/v1/support", support::router())
        .nest("/api/v1/system-logs", system_logs::router())
        .nest("/api/v1/tasks", tasks::router())
        .nest("/api/v1/tracking", tracking::router())
        .nest("/api/v1/users", users::router())
        .nest("/api/v1/vehicles", vehicles::router())
        .nest("/api/v1/logs", audit::router())
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit::middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let bind_addr = state.config.bind_addr()?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!("Rust API listening on {}", bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct HealthDetailsResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "rust-api",
    })
}

async fn health_details(State(state): State<AppState>) -> Json<HealthDetailsResponse> {
    let database = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        "ok"
    } else {
        "error"
    };

    let status = if database == "ok" { "ok" } else { "degraded" };
    Json(HealthDetailsResponse {
        status,
        service: "rust-api",
        database,
        version: env!("CARGO_PKG_VERSION"),
    })
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    let origins = config
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
}
