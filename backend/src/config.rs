use anyhow::{Context, Result};
use std::{env, net::SocketAddr, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub storage_root: PathBuf,
    pub rate_limit_per_minute: usize,
    pub cors_allowed_origins: Vec<String>,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub access_token_expire_minutes: i64,
    pub refresh_token_expire_days: i64,
    pub bootstrap_admin_email: String,
    pub bootstrap_admin_password: String,
    pub ai_provider: String,
    pub ai_model: String,
    pub ai_api_key: Option<String>,
    pub ai_base_url: Option<String>,
    pub ai_timeout_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("BACKEND_PORT must be a number")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            storage_root: PathBuf::from(
                env::var("STORAGE_ROOT").unwrap_or_else(|_| "./storage".to_string()),
            ),
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .context("RATE_LIMIT_PER_MINUTE must be a number")?,
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_string())
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_string)
                .collect(),
            jwt_access_secret: env::var("JWT_ACCESS_SECRET")
                .context("JWT_ACCESS_SECRET is required")?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .context("JWT_REFRESH_SECRET is required")?,
            access_token_expire_minutes: env::var("ACCESS_TOKEN_EXPIRE_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("ACCESS_TOKEN_EXPIRE_MINUTES must be a number")?,
            refresh_token_expire_days: env::var("REFRESH_TOKEN_EXPIRE_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("REFRESH_TOKEN_EXPIRE_DAYS must be a number")?,
            bootstrap_admin_email: env::var("BOOTSTRAP_ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@tuluklar.local".to_string()),
            bootstrap_admin_password: env::var("BOOTSTRAP_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "Admin12345!".to_string()),
            ai_provider: env::var("AI_PROVIDER").unwrap_or_else(|_| "disabled".to_string()),
            ai_model: env::var("AI_MODEL").unwrap_or_else(|_| "not-configured".to_string()),
            ai_api_key: env::var("AI_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            ai_base_url: env::var("AI_BASE_URL")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            ai_timeout_seconds: env::var("AI_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("AI_TIMEOUT_SECONDS must be a number")?,
        })
    }

    pub fn bind_addr(&self) -> Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .context("BACKEND_HOST/BACKEND_PORT must form a valid socket address")
    }
}
