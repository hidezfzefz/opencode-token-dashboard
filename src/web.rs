use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::DateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use tera::{Context, Tera};

use crate::chart;
use crate::csv;
use crate::format::{format_compact, format_number, format_percent};
use crate::usage::{self, SessionTotals, UsageSnapshot};

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub db_path: PathBuf,
    pub refresh_seconds: i64,
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
    let path = state.db_path.clone();
    match tokio::task::spawn_blocking(move || usage::get_usage(&path)).await {
        Ok(result) => result,
        Err(error) => Err(format!("Could not load usage: {error}")),
    }
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

async fn sessions(State(state): State<AppState>, Query(query): Query<SessionsQuery>) -> Response {
    let snapshot = match load_usage(&state).await {
        Ok(snapshot) => snapshot,
        Err(error) => return error_response(&state, error),
    };
    let q = query.q.unwrap_or_default();
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

    let valid_keys = [
        "label",
        "updatedAt",
        "input",
        "output",
        "reasoning",
        "cacheRead",
        "fresh",
        "total",
    ];
    let sort = query.sort.unwrap_or_else(|| "total".to_string());
    let sort = if valid_keys.contains(&sort.as_str()) {
        sort
    } else {
        "total".to_string()
    };
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    sort_sessions(&mut rows, &sort, dir);

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
    let q = query.q.unwrap_or_default();
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
    let valid_keys = [
        "label",
        "updatedAt",
        "input",
        "output",
        "reasoning",
        "cacheRead",
        "fresh",
        "total",
    ];
    let sort = query.sort.unwrap_or_else(|| "total".to_string());
    let sort = if valid_keys.contains(&sort.as_str()) {
        sort
    } else {
        "total".to_string()
    };
    sort_sessions(
        &mut rows,
        &sort,
        &query.dir.unwrap_or_else(|| "desc".to_string()),
    );
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
        .fallback(not_found)
        .layer(middleware::from_fn(security_headers))
        .with_state(state)
}
