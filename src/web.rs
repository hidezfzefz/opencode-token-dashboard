use std::path::PathBuf;
use std::sync::Arc;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use axum::body::Body;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use tera::{Context, Tera};

use crate::chart;
use crate::csv;
use crate::format::{format_compact, format_number, format_percent};
use crate::usage::{self, SessionTotals, UsageSnapshot};

pub fn templates() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_template("base.html", include_str!("../templates/base.html"))
        .expect("base template");
    tera.add_raw_template("overview.html", include_str!("../templates/overview.html"))
        .expect("overview template");
    tera.add_raw_template("sessions.html", include_str!("../templates/sessions.html"))
        .expect("sessions template");
    tera.add_raw_template("models.html", include_str!("../templates/models.html"))
        .expect("models template");
    tera.add_raw_template("projects.html", include_str!("../templates/projects.html"))
        .expect("projects template");
    tera.add_raw_template("error.html", include_str!("../templates/error.html"))
        .expect("error template");
    tera.add_raw_template(
        "not_found.html",
        include_str!("../templates/not_found.html"),
    )
    .expect("not found template");
    tera
}

type SnapshotCache = Arc<Mutex<Option<(Vec<(u64, u128)>, UsageSnapshot)>>>;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub db_path: PathBuf,
    pub refresh_seconds: i64,
    pub pricing: crate::cost::Pricing,
    pub config_path: PathBuf,
    pub cache: SnapshotCache,
}

#[derive(Deserialize)]
pub struct OverviewQuery {
    range: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionsQuery {
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
}

fn render(
    state: &AppState,
    template: &str,
    title: &str,
    active: &str,
    context: &mut Context,
    status: StatusCode,
) -> Response {
    render_with_refresh(
        state,
        template,
        title,
        active,
        context,
        status,
        state.refresh_seconds,
    )
}

fn render_with_refresh(
    state: &AppState,
    template: &str,
    title: &str,
    active: &str,
    context: &mut Context,
    status: StatusCode,
    refresh_seconds: i64,
) -> Response {
    context.insert("title", title);
    context.insert("active", active);
    context.insert("refresh_seconds", &refresh_seconds);
    context.insert("cost_configured", &!state.pricing.models.is_empty());
    match state.tera.render(template, context) {
        Ok(html) => (status, Html(html)).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!("Template error: {error}")),
        )
            .into_response(),
    }
}

fn error_response(state: &AppState, message: String) -> Response {
    let mut context = Context::new();
    context.insert("message", &message);
    render(
        state,
        "error.html",
        "Could not load token usage",
        "",
        &mut context,
        StatusCode::INTERNAL_SERVER_ERROR,
    )
}

async fn load_usage(state: &AppState) -> Result<UsageSnapshot, String> {
    let fingerprint = usage::db_fingerprint(&state.db_path);
    if let Ok(cache) = state.cache.lock() {
        if let Some((cached_fingerprint, snapshot)) = cache.as_ref() {
            if *cached_fingerprint == fingerprint {
                return Ok(snapshot.clone());
            }
        }
    }

    let path = state.db_path.clone();
    let result = match tokio::task::spawn_blocking(move || usage::get_usage(&path)).await {
        Ok(result) => result,
        Err(error) => Err(format!("Could not load usage: {error}")),
    };

    if let Ok(snapshot) = &result {
        if let Ok(mut cache) = state.cache.lock() {
            *cache = Some((fingerprint, snapshot.clone()));
        }
    }
    result
}

fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn short_date(millis: i64) -> String {
    DateTime::from_timestamp_millis(millis)
        .map(|date| date.format("%m/%d/%Y").to_string())
        .unwrap_or_default()
}

fn days_for<'a>(snapshot: &'a UsageSnapshot, range: &str) -> &'a [crate::usage::DayTotals] {
    if range == "all" {
        &snapshot.per_day
    } else {
        let len = snapshot.per_day.len().min(30);
        &snapshot.per_day[snapshot.per_day.len() - len..]
    }
}

fn daily_view(rows: &[crate::usage::DayTotals]) -> Vec<Value> {
    rows.iter()
        .rev()
        .map(|row| {
            json!({
                "day": row.day,
                "fresh": format_compact(row.fresh),
                "cache_read": format_compact(row.cache_read),
                "total": format_number(row.total),
            })
        })
        .collect()
}

fn session_view(rows: &[SessionTotals]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "label": row.label,
                "model": row.model,
                "fresh": format_compact(row.fresh),
                "cache_read": format_compact(row.cache_read),
                "total": format_number(row.total),
            })
        })
        .collect()
}

async fn overview(State(state): State<AppState>, Query(query): Query<OverviewQuery>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let range = if query.range.as_deref() == Some("all") {
        "all"
    } else {
        "30d"
    };
    let days = days_for(&snapshot, range);
    let mut context = Context::new();
    context.insert("db_path", &snapshot.db_path);
    context.insert("today_total", &format_compact(snapshot.today.total));
    context.insert("today_total_exact", &format_number(snapshot.today.total));
    context.insert("today_fresh", &format_compact(snapshot.today.fresh));
    context.insert("today_fresh_exact", &format_number(snapshot.today.fresh));
    context.insert(
        "today_percent",
        &format_percent(snapshot.today.fresh, snapshot.today.total),
    );
    context.insert("all_total", &format_compact(snapshot.all_time.total));
    context.insert("all_total_exact", &format_number(snapshot.all_time.total));
    context.insert("all_fresh", &format_compact(snapshot.all_time.fresh));
    context.insert("all_fresh_exact", &format_number(snapshot.all_time.fresh));
    context.insert(
        "all_percent",
        &format_percent(snapshot.all_time.fresh, snapshot.all_time.total),
    );
    context.insert(
        "message_count",
        &format_number(snapshot.message_count as i64),
    );
    context.insert("yesterday_total", &format_compact(snapshot.yesterday.total));
    context.insert(
        "seven_day_average",
        &format_compact(snapshot.seven_day_average.total),
    );
    context.insert("range", range);
    context.insert("chart_svg", &chart::area_chart_svg(days, 1000.0, 6));
    context.insert("chart_mobile_svg", &chart::area_chart_svg(days, 390.0, 3));
    context.insert("daily_rows", &daily_view(days));
    context.insert(
        "session_rows",
        &session_view(&snapshot.sessions[..snapshot.sessions.len().min(8)]),
    );
    let costs = crate::cost::estimate(&snapshot, &state.pricing);
    let cost_rows: Vec<Value> = costs
        .rows
        .iter()
        .map(|row| {
            json!({
                "model": row.model,
                "messages": format_number(row.messages),
                "total": crate::cost::format_money(row.total_usd),
            })
        })
        .collect();
    context.insert("cost_rows", &cost_rows);
    context.insert("cost_total", &crate::cost::format_money(costs.total_usd));
    render(
        &state,
        "overview.html",
        "Token usage",
        "overview",
        &mut context,
        StatusCode::OK,
    )
}

fn sort_sessions(rows: &mut [SessionTotals], key: &str, direction: &str) {
    let ascending = direction == "asc";
    rows.sort_by(|a, b| {
        let order = match key {
            "label" => a.label.cmp(&b.label),
            "updatedAt" => a.updated_at.cmp(&b.updated_at),
            "input" => a.input.cmp(&b.input),
            "output" => a.output.cmp(&b.output),
            "reasoning" => a.reasoning.cmp(&b.reasoning),
            "cacheRead" => a.cache_read.cmp(&b.cache_read),
            "fresh" => a.fresh.cmp(&b.fresh),
            _ => a.total.cmp(&b.total),
        };
        if ascending {
            order
        } else {
            order.reverse()
        }
    });
}

const SORT_KEYS: [&str; 8] = [
    "label",
    "updatedAt",
    "input",
    "output",
    "reasoning",
    "cacheRead",
    "fresh",
    "total",
];

fn filtered_sessions(
    snapshot: &UsageSnapshot,
    query: &SessionsQuery,
) -> (String, String, &'static str, Vec<SessionTotals>) {
    let q = query.q.clone().unwrap_or_default();
    let needle = q.trim().to_lowercase();
    let mut rows: Vec<SessionTotals> = snapshot
        .sessions
        .iter()
        .filter(|row| {
            needle.is_empty()
                || format!("{} {} {}", row.label, row.model, row.project)
                    .to_lowercase()
                    .contains(&needle)
        })
        .cloned()
        .collect();

    let mut sort = query.sort.clone().unwrap_or_else(|| "total".to_string());
    if !SORT_KEYS.contains(&sort.as_str()) {
        sort = "total".to_string();
    }
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    sort_sessions(&mut rows, &sort, dir);
    (q, sort, dir, rows)
}

async fn sessions(State(state): State<AppState>, Query(query): Query<SessionsQuery>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let (q, sort, dir, view_rows) = filtered_sessions(&snapshot, &query);
    let rows = view_rows;

    let encoded_q = url_encode(&q);
    let columns: Vec<Value> = [
        ("Session", "label", false),
        ("Last active", "updatedAt", true),
        ("Input", "input", true),
        ("Output", "output", true),
        ("Reasoning", "reasoning", true),
        ("Cache read", "cacheRead", true),
        ("Fresh", "fresh", true),
        ("Total", "total", true),
    ]
    .iter()
    .map(|(label, key, numeric)| {
        let next_dir = if sort == *key && dir == "desc" {
            "asc"
        } else {
            "desc"
        };
        json!({
            "label": label,
            "numeric": numeric,
            "href": format!("/sessions?q={encoded_q}&sort={key}&dir={next_dir}"),
        })
    })
    .collect();

    let view: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "label": row.label,
                "updated_at": short_date(row.updated_at),
                "input": format_compact(row.input),
                "output": format_compact(row.output),
                "reasoning": format_compact(row.reasoning),
                "cache_read": format_compact(row.cache_read),
                "fresh": format_compact(row.fresh),
                "total": format_number(row.total),
            })
        })
        .collect();

    let mut query_string = format!("q={encoded_q}&sort={}&dir={dir}", url_encode(&sort));
    if query_string.starts_with("q=&") {
        query_string = query_string[3..].to_string();
    }

    let mut context = Context::new();
    context.insert("q", &q);
    context.insert("query_string", &query_string);
    context.insert("rows", &view);
    context.insert("columns", &columns);
    context.insert(
        "session_count",
        &format_number(snapshot.sessions.len() as i64),
    );
    context.insert("total_sessions", &snapshot.sessions.len());
    // A full page reload would wipe what the user is typing in the search box.
    let refresh = if q.trim().is_empty() {
        state.refresh_seconds
    } else {
        0
    };
    render_with_refresh(
        &state,
        "sessions.html",
        "Sessions",
        "sessions",
        &mut context,
        StatusCode::OK,
        refresh,
    )
}

fn breakdown_view(rows: &[crate::usage::NamedTotals], all_time: i64) -> Vec<Value> {
    let max = rows.first().map(|row| row.total).unwrap_or(0);
    rows.iter()
        .map(|row| {
            let width = if max > 0 {
                ((row.total as f64 / max as f64) * 100.0).round().max(2.0) as i64
            } else {
                0
            };
            json!({
                "label": row.label,
                "messages": format_number(row.messages),
                "fresh": format_compact(row.fresh),
                "cache_read": format_compact(row.cache_read),
                "total": format_number(row.total),
                "share_width": width,
                "percent": format_percent(row.total, all_time),
            })
        })
        .collect()
}

async fn models(State(state): State<AppState>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let mut context = Context::new();
    context.insert(
        "rows",
        &breakdown_view(&snapshot.models, snapshot.all_time.total),
    );
    context.insert("row_count", &snapshot.models.len());
    render(
        &state,
        "models.html",
        "Models",
        "models",
        &mut context,
        StatusCode::OK,
    )
}

async fn projects(State(state): State<AppState>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let mut context = Context::new();
    context.insert(
        "rows",
        &breakdown_view(&snapshot.projects, snapshot.all_time.total),
    );
    context.insert("row_count", &snapshot.projects.len());
    render(
        &state,
        "projects.html",
        "Projects",
        "projects",
        &mut context,
        StatusCode::OK,
    )
}

fn csv_response(content: String, filename: &str) -> Response {
    let mut response = Response::new(Body::from(content));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

async fn sessions_csv(
    State(state): State<AppState>,
    Query(query): Query<SessionsQuery>,
) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let (_q, _sort, _dir, rows) = filtered_sessions(&snapshot, &query);
    csv_response(
        csv::sessions_to_csv(&rows),
        "opencode-token-usage-sessions.csv",
    )
}

async fn daily_csv(State(state): State<AppState>, Query(query): Query<OverviewQuery>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let range = if query.range.as_deref() == Some("all") {
        "all"
    } else {
        "30d"
    };
    let days = days_for(&snapshot, range);
    let mut ordered: Vec<crate::usage::DayTotals> = days.to_vec();
    ordered.reverse();
    csv_response(
        csv::daily_to_csv(&ordered),
        "opencode-token-usage-daily.csv",
    )
}

fn json_error(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "error": { "code": code, "message": message.into() } })),
    )
        .into_response()
}

fn api_data_error(error: String) -> Response {
    json_error(StatusCode::INTERNAL_SERVER_ERROR, "database_error", error)
}

async fn api_usage(State(state): State<AppState>) -> Response {
    match load_usage(&state).await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(error) => api_data_error(error),
    }
}

async fn api_daily(State(state): State<AppState>, Query(query): Query<OverviewQuery>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return api_data_error(error),
    };
    let range = if query.range.as_deref() == Some("all") {
        "all"
    } else {
        "30d"
    };
    let mut rows = days_for(&snapshot, range).to_vec();
    rows.reverse();
    Json(rows).into_response()
}

async fn api_sessions(
    State(state): State<AppState>,
    Query(query): Query<SessionsQuery>,
) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return api_data_error(error),
    };
    let (_q, _sort, _dir, rows) = filtered_sessions(&snapshot, &query);
    Json(rows).into_response()
}

async fn api_costs(State(state): State<AppState>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return api_data_error(error),
    };
    Json(crate::cost::estimate(&snapshot, &state.pricing)).into_response()
}

async fn api_get_settings(State(state): State<AppState>) -> Response {
    Json(crate::settings::load_from(&state.config_path)).into_response()
}

async fn api_put_settings(
    State(state): State<AppState>,
    payload: Result<Json<crate::settings::Settings>, JsonRejection>,
) -> Response {
    let settings = match payload {
        Ok(Json(settings)) => settings,
        Err(_) => {
            return json_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_settings",
                "Malformed settings body",
            )
        }
    };
    if !settings.is_valid() {
        return json_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_settings",
            "autoSyncSeconds must be one of 0, 1, 5, 15 or 60",
        );
    }
    match crate::settings::save_to(&state.config_path, &settings) {
        Ok(()) => Json(settings).into_response(),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "write_error", error),
    }
}

async fn health_live() -> Response {
    Json(json!({ "status": "ok" })).into_response()
}

async fn health_ready(State(state): State<AppState>) -> Response {
    let path = state.db_path.clone();
    let ready = tokio::task::spawn_blocking(move || crate::usage::get_usage(&path).is_ok())
        .await
        .unwrap_or(false);
    if ready {
        Json(json!({ "status": "ready", "checks": { "database": "ok" } })).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "not_ready", "checks": { "database": "error" } })),
        )
            .into_response()
    }
}

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

async fn request_logging(request: Request, next: Next) -> Response {
    let id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = Instant::now();
    let mut response = next.run(request).await;
    let status = response.status();
    if let Ok(value) = HeaderValue::from_str(&id.to_string()) {
        response.headers_mut().insert("x-request-id", value);
    }
    eprintln!(
        "request_id={id} method={method} path={path} status={} ms={}",
        status.as_u16(),
        started.elapsed().as_millis()
    );
    response
}

async fn style_css() -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )],
        include_str!("../static/style.css"),
    )
        .into_response()
}

async fn icon_svg() -> Response {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/svg+xml"),
        )],
        include_str!("../static/icon.svg"),
    )
        .into_response()
}

async fn not_found(State(state): State<AppState>) -> Response {
    let mut context = Context::new();
    render(
        &state,
        "not_found.html",
        "Page not found",
        "",
        &mut context,
        StatusCode::NOT_FOUND,
    )
}

pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(overview))
        .route("/sessions", get(sessions))
        .route("/models", get(models))
        .route("/projects", get(projects))
        .route("/sessions.csv", get(sessions_csv))
        .route("/daily.csv", get(daily_csv))
        .route("/static/style.css", get(style_css))
        .route("/icon.svg", get(icon_svg))
        .route("/api/v1/usage", get(api_usage))
        .route("/api/v1/usage/daily", get(api_daily))
        .route("/api/v1/usage/sessions", get(api_sessions))
        .route("/api/v1/usage/costs", get(api_costs))
        .route(
            "/api/v1/settings",
            get(api_get_settings).put(api_put_settings),
        )
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .fallback(not_found)
        .layer(middleware::from_fn(request_logging))
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}

#[cfg(test)]
mod api_tests {
    use super::*;
    use axum::body::to_bytes;
    use rusqlite::Connection;
    use tower::ServiceExt;

    struct Fixture {
        dir: PathBuf,
        db_path: PathBuf,
        config_path: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("otd-api-{nanos}"));
            std::fs::create_dir_all(&dir).unwrap();
            let db_path = dir.join("usage.db");
            let config_path = dir.join("config.json");

            let db = Connection::open(&db_path).unwrap();
            db.execute_batch(
                "CREATE TABLE session_message (id TEXT PRIMARY KEY, session_id TEXT, type TEXT, time_created INTEGER, data TEXT);
                 CREATE TABLE session_v2 (id TEXT PRIMARY KEY, title TEXT, directory TEXT, project_id TEXT);
                 CREATE TABLE project (id TEXT PRIMARY KEY, name TEXT, worktree TEXT);",
            )
            .unwrap();
            db.execute(
                "INSERT INTO project VALUES ('p1','alpha','/work/alpha')",
                [],
            )
            .unwrap();
            db.execute(
                "INSERT INTO session_v2 VALUES ('s1','First session','/work/alpha','p1')",
                [],
            )
            .unwrap();
            db.execute(
                "INSERT INTO session_v2 VALUES ('s2','Second session','/work/alpha','p1')",
                [],
            )
            .unwrap();
            let message = |input: i64, output: i64, cache: i64| {
                format!(
                    "{{\"model\":{{\"id\":\"gpt\",\"providerID\":\"test\",\"variant\":\"\"}},\"tokens\":{{\"input\":{input},\"output\":{output},\"reasoning\":0,\"cache\":{{\"read\":{cache},\"write\":0}}}}}}"
                )
            };
            let now = chrono::Local::now();
            let today = now
                .date_naive()
                .and_hms_opt(10, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .timestamp_millis();
            let yesterday = (now.date_naive() - chrono::Duration::days(1))
                .and_hms_opt(10, 0, 0)
                .unwrap()
                .and_local_timezone(chrono::Local)
                .unwrap()
                .timestamp_millis();
            db.execute(
                "INSERT INTO session_message VALUES ('m1','s1','assistant',?1,?2)",
                rusqlite::params![today, message(10, 5, 100)],
            )
            .unwrap();
            db.execute(
                "INSERT INTO session_message VALUES ('m2','s2','assistant',?1,?2)",
                rusqlite::params![yesterday, message(20, 6, 200)],
            )
            .unwrap();
            db.execute(
                "INSERT INTO session_message VALUES ('m3','s1','user',?1,'{}')",
                rusqlite::params![today],
            )
            .unwrap();
            drop(db);

            Self {
                dir,
                db_path,
                config_path,
            }
        }

        fn state(&self) -> AppState {
            AppState {
                tera: Arc::new(templates()),
                db_path: self.db_path.clone(),
                refresh_seconds: 15,
                pricing: crate::cost::Pricing::default(),
                config_path: self.config_path.clone(),
                cache: Arc::new(Mutex::new(None)),
            }
        }

        fn app(&self) -> Router {
            router(self.state())
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    async fn call(app: &Router, method: &str, uri: &str, body: Body) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        if method != "GET" {
            builder = builder.header("content-type", "application/json");
        }
        let response = app
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    async fn get(app: &Router, uri: &str) -> (StatusCode, Value) {
        call(app, "GET", uri, Body::empty()).await
    }

    #[tokio::test]
    async fn usage_endpoint_returns_the_snapshot() {
        let fixture = Fixture::new();
        let (status, body) = get(&fixture.app(), "/api/v1/usage").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["messageCount"], 2);
        assert_eq!(body["allTime"]["total"], 341);
        assert_eq!(body["allTime"]["fresh"], 41);
        assert_eq!(body["perDay"].as_array().unwrap().len(), 2);
        assert!(body["dbPath"].as_str().unwrap().ends_with("usage.db"));
    }

    #[tokio::test]
    async fn daily_endpoint_uses_requested_range_and_newest_first() {
        let fixture = Fixture::new();
        let (status, body) = get(&fixture.app(), "/api/v1/usage/daily?range=all").await;
        assert_eq!(status, StatusCode::OK);
        let rows = body.as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows[0]["day"].as_str().unwrap() > rows[1]["day"].as_str().unwrap());
    }

    #[tokio::test]
    async fn sessions_endpoint_filters_and_sorts() {
        let fixture = Fixture::new();
        let app = fixture.app();
        let (status, body) = get(&app, "/api/v1/usage/sessions?q=first").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["label"], "First session");

        let (_, body) = get(&app, "/api/v1/usage/sessions?sort=input&dir=asc").await;
        assert_eq!(body[0]["input"], 10);
        let (_, body) = get(&app, "/api/v1/usage/sessions?sort=input&dir=desc").await;
        assert_eq!(body[0]["input"], 20);
    }

    #[tokio::test]
    async fn costs_endpoint_reports_unconfigured_pricing() {
        let fixture = Fixture::new();
        let (status, body) = get(&fixture.app(), "/api/v1/usage/costs").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["configured"], false);
        assert_eq!(body["totalUsd"], 0.0);
        assert_eq!(body["rows"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn settings_endpoints_round_trip_and_reject_invalid_values() {
        let fixture = Fixture::new();
        let app = fixture.app();
        let (status, body) = get(&app, "/api/v1/settings").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["autoSyncSeconds"], 1);

        let body = Body::from(r#"{"autoSyncSeconds":15,"allDays":true}"#);
        let (status, saved) = call(&app, "PUT", "/api/v1/settings", body).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(saved["autoSyncSeconds"], 15);
        assert_eq!(saved["allDays"], true);

        let (status, body) = get(&app, "/api/v1/settings").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["autoSyncSeconds"], 15);

        let bad = Body::from(r#"{"autoSyncSeconds":7,"allDays":false}"#);
        let (status, error) = call(&app, "PUT", "/api/v1/settings", bad).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(error["error"]["code"], "invalid_settings");

        let unknown = Body::from(r#"{"autoSyncSeconds":1,"allDays":false,"extra":1}"#);
        let (status, _) = call(&app, "PUT", "/api/v1/settings", unknown).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn health_endpoints_report_liveness_and_readiness() {
        let fixture = Fixture::new();
        let app = fixture.app();
        let (status, body) = get(&app, "/health/live").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "ok");
        let (status, body) = get(&app, "/health/ready").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["checks"]["database"], "ok");
    }

    #[tokio::test]
    async fn missing_database_returns_agreed_error_format() {
        let fixture = Fixture::new();
        let mut state = fixture.state();
        state.db_path = fixture.dir.join("missing.db");
        let (status, body) = get(&router(state), "/api/v1/usage").await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["error"]["code"], "database_error");
        assert!(body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not found"));
    }
}
