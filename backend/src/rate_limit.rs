use crate::AppState;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    fn allow(&self, key: String, limit: usize) -> bool {
        let mut windows = match self.windows.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        let now = Instant::now();
        let window = windows.entry(key).or_insert_with(VecDeque::new);
        while window
            .front()
            .map(|created_at| now.duration_since(*created_at) > Duration::from_secs(60))
            .unwrap_or(false)
        {
            window.pop_front();
        }
        if window.len() >= limit {
            return false;
        }
        window.push_back(now);
        true
    }
}

pub async fn middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local")
        .to_string();

    if !state
        .rate_limiter
        .allow(key, state.config.rate_limit_per_minute)
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "error": "rate limit exceeded" })),
        )
            .into_response();
    }

    next.run(request).await
}
