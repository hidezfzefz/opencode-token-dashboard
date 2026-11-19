use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Local, NaiveDate};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

#[derive(Clone, Copy, Default)]
pub struct Counts {
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
}

impl Counts {
    fn add(&mut self, other: &Counts) {
        self.input += other.input;
        self.output += other.output;
        self.reasoning += other.reasoning;
        self.cache_read += other.cache_read;
        self.cache_write += other.cache_write;
    }
}

fn totals_from(counts: &Counts) -> Totals {
    Totals {
        input: counts.input,
        output: counts.output,
        reasoning: counts.reasoning,
        cache_read: counts.cache_read,
        cache_write: counts.cache_write,
        fresh: counts.input + counts.output + counts.reasoning,
        total: counts.input
            + counts.output
            + counts.reasoning
            + counts.cache_read
            + counts.cache_write,
    }
}

#[derive(Clone, Default, Serialize)]
pub struct Totals {
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub fresh: i64,
    pub total: i64,
}

#[derive(Clone, Serialize)]
pub struct DayTotals {
    pub day: String,
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub fresh: i64,
    pub total: i64,
}

impl DayTotals {
    fn new(day: String, counts: &Counts) -> Self {
        let t = totals_from(counts);
        Self {
            day,
            input: t.input,
            output: t.output,
            reasoning: t.reasoning,
            cache_read: t.cache_read,
            cache_write: t.cache_write,
            fresh: t.fresh,
            total: t.total,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct NamedTotals {
    pub key: String,
    pub label: String,
    pub messages: i64,
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub fresh: i64,
    pub total: i64,
}

impl NamedTotals {
    fn new(key: String, label: String, counts: &Counts) -> Self {
        let t = totals_from(counts);
        Self {
            key,
            label,
            messages: 0,
            input: t.input,
            output: t.output,
            reasoning: t.reasoning,
            cache_read: t.cache_read,
            cache_write: t.cache_write,
            fresh: t.fresh,
            total: t.total,
        }
    }

    fn finish(&mut self, counts: &Counts) {
        let t = totals_from(counts);
        self.input = t.input;
        self.output = t.output;
        self.reasoning = t.reasoning;
        self.cache_read = t.cache_read;
        self.cache_write = t.cache_write;
        self.fresh = t.fresh;
        self.total = t.total;
    }
}

#[derive(Clone, Serialize)]
pub struct SessionTotals {
    pub key: String,
    pub label: String,
    pub model: String,
    pub project: String,
    pub updated_at: i64,
    pub messages: i64,
    pub input: i64,
    pub output: i64,
    pub reasoning: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    pub fresh: i64,
    pub total: i64,
}

impl SessionTotals {
    fn finish(&mut self, counts: &Counts) {
        let t = totals_from(counts);
        self.input = t.input;
        self.output = t.output;
        self.reasoning = t.reasoning;
        self.cache_read = t.cache_read;
        self.cache_write = t.cache_write;
        self.fresh = t.fresh;
        self.total = t.total;
    }
}

#[derive(Clone, Serialize)]
pub struct UsageSnapshot {
    pub db_path: String,
    pub message_count: usize,
    pub today: Totals,
    pub yesterday: Totals,
    pub seven_day_average: Totals,
    pub all_time: Totals,
    pub per_day: Vec<DayTotals>,
    pub sessions: Vec<SessionTotals>,
    pub models: Vec<NamedTotals>,
    pub projects: Vec<NamedTotals>,
}

pub struct UsageRow {
    session_id: String,
    time_created: i64,
    model: String,
    session_title: String,
    project_id: String,
    project_label: String,
    counts: Counts,
}

pub fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("OPENCODE_DB_PATH") {
        if !path.is_empty() {
            return PathBuf::from(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = std::env::var("LOCALAPPDATA") {
            if !path.is_empty() {
                return PathBuf::from(path).join("opencode").join("opencode.db");
            }
        }
        if let Ok(path) = std::env::var("APPDATA") {
            if !path.is_empty() {
                return PathBuf::from(path).join("opencode").join("opencode.db");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("opencode")
                    .join("opencode.db");
            }
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("opencode")
        .join("opencode.db")
}

/// A cheap fingerprint of the database files. OpenCode writes most changes to
/// the WAL file, so all three files are checked.
pub fn db_fingerprint(path: &Path) -> Vec<(u64, u128)> {
    let mut candidates = vec![path.to_path_buf()];
    let name = path.to_string_lossy();
    candidates.push(PathBuf::from(format!("{name}-wal")));
    candidates.push(PathBuf::from(format!("{name}-shm")));

    let mut result = Vec::new();
    for candidate in candidates {
        if let Ok(metadata) = std::fs::metadata(candidate) {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            result.push((metadata.len(), modified));
        }
    }
    result
}

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if !path.is_file() {
        return Err(format!("OpenCode database not found at {}", path.display()));
    }
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Could not open {}: {e}", path.display()))
}

pub fn read_rows(path: &Path) -> Result<Vec<UsageRow>, String> {
    let db = open_db(path)?;
    let sql = r#"
        SELECT
            m.session_id AS session_id,
            m.time_created AS time_created,
            COALESCE(json_extract(m.data, '$.model.id'), 'unknown') AS model_id,
            COALESCE(json_extract(m.data, '$.model.providerID'), 'unknown') AS provider_id,
            COALESCE(json_extract(m.data, '$.model.variant'), '') AS variant,
            CAST(COALESCE(json_extract(m.data, '$.tokens.input'), 0) AS INTEGER) AS input,
            CAST(COALESCE(json_extract(m.data, '$.tokens.output'), 0) AS INTEGER) AS output,
            CAST(COALESCE(json_extract(m.data, '$.tokens.reasoning'), 0) AS INTEGER) AS reasoning,
            CAST(COALESCE(json_extract(m.data, '$.tokens.cache.read'), 0) AS INTEGER) AS cache_read,
            CAST(COALESCE(json_extract(m.data, '$.tokens.cache.write'), 0) AS INTEGER) AS cache_write,
            COALESCE(NULLIF(s.title, ''), m.session_id) AS session_title,
            COALESCE(p.id, 'none') AS project_id,
            COALESCE(NULLIF(p.name, ''), NULLIF(p.worktree, ''), s.directory, 'Other') AS project_label
        FROM session_message m
        LEFT JOIN session_v2 s ON s.id = m.session_id
        LEFT JOIN project p ON p.id = s.project_id
        WHERE m.type = 'assistant'
        ORDER BY m.time_created ASC
    "#;
    let mut stmt = db
        .prepare(sql)
        .map_err(|e| format!("Database query failed: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            let model_id: String = row.get(2)?;
            let provider_id: String = row.get(3)?;
            let variant: String = row.get(4)?;
            let model = if variant.is_empty() {
                format!("{provider_id}/{model_id}")
            } else {
                format!("{provider_id}/{model_id}#{variant}")
            };
            Ok(UsageRow {
                session_id: row.get(0)?,
                time_created: row.get(1)?,
                model,
                session_title: row.get(10)?,
                project_id: row.get(11)?,
                project_label: row.get(12)?,
                counts: Counts {
                    input: row.get(5)?,
                    output: row.get(6)?,
                    reasoning: row.get(7)?,
                    cache_read: row.get(8)?,
                    cache_write: row.get(9)?,
                },
            })
        })
        .map_err(|e| format!("Database query failed: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("Could not read a usage row: {e}"))?);
    }
    Ok(result)
}

fn day_key(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Filter sessions by a free-text query, an exact model and an exact project.
/// Empty filter values are ignored.
pub fn filter_sessions(
    rows: &[SessionTotals],
    query: &str,
    model: &str,
    project: &str,
) -> Vec<SessionTotals> {
    let needle = query.trim().to_lowercase();
    rows.iter()
        .filter(|row| {
            let text_match = needle.is_empty()
                || format!("{} {} {}", row.label, row.model, row.project)
                    .to_lowercase()
                    .contains(&needle);
            let model_match = model.is_empty() || row.model == model;
            let project_match = project.is_empty() || row.project == project;
            text_match && model_match && project_match
        })
        .cloned()
        .collect()
}

pub fn aggregate(rows: Vec<UsageRow>, path: &Path, now: DateTime<Local>) -> UsageSnapshot {
    let message_count = rows.len();
    let today_date = now.date_naive();
    let today_key = day_key(today_date);

    let mut all_time = Counts::default();
    let mut today = Counts::default();
    let mut days: BTreeMap<String, Counts> = BTreeMap::new();
    let mut sessions: BTreeMap<String, (SessionTotals, Counts)> = BTreeMap::new();
    let mut models: BTreeMap<String, (NamedTotals, Counts)> = BTreeMap::new();
    let mut projects: BTreeMap<String, (NamedTotals, Counts)> = BTreeMap::new();

    for row in rows {
        all_time.add(&row.counts);
        let key = chrono::DateTime::from_timestamp_millis(row.time_created)
            .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string();
        days.entry(key.clone()).or_default().add(&row.counts);
        if key == today_key {
            today.add(&row.counts);
        }

        let session_key = if row.session_id.is_empty() {
            "unknown".to_string()
        } else {
            row.session_id.clone()
        };
        let entry = sessions.entry(session_key.clone()).or_insert_with(|| {
            (
                SessionTotals {
                    key: session_key.clone(),
                    label: if row.session_title.is_empty() {
                        session_key.clone()
                    } else {
                        row.session_title.clone()
                    },
                    model: row.model.clone(),
                    project: row.project_label.clone(),
                    updated_at: 0,
                    messages: 0,
                    input: 0,
                    output: 0,
                    reasoning: 0,
                    cache_read: 0,
                    cache_write: 0,
                    fresh: 0,
                    total: 0,
                },
                Counts::default(),
            )
        });
        entry.1.add(&row.counts);
        entry.0.messages += 1;
        entry.0.updated_at = entry.0.updated_at.max(row.time_created);

        let model_entry = models.entry(row.model.clone()).or_insert_with(|| {
            (
                NamedTotals::new(row.model.clone(), row.model.clone(), &Counts::default()),
                Counts::default(),
            )
        });
        model_entry.1.add(&row.counts);
        model_entry.0.messages += 1;

        let project_entry = projects.entry(row.project_id.clone()).or_insert_with(|| {
            (
                NamedTotals::new(
                    row.project_id.clone(),
                    row.project_label.clone(),
                    &Counts::default(),
                ),
                Counts::default(),
            )
        });
        project_entry.1.add(&row.counts);
        project_entry.0.messages += 1;
    }

    let mut yesterday = Counts::default();
    let mut seven_days = Counts::default();
    for offset in 1..=7 {
        let key = day_key(today_date - Duration::days(offset));
        if let Some(counts) = days.get(&key) {
            if offset == 1 {
                yesterday.add(counts);
            }
            seven_days.add(counts);
        }
    }
    let average = Counts {
        input: (seven_days.input as f64 / 7.0).round() as i64,
        output: (seven_days.output as f64 / 7.0).round() as i64,
        reasoning: (seven_days.reasoning as f64 / 7.0).round() as i64,
        cache_read: (seven_days.cache_read as f64 / 7.0).round() as i64,
        cache_write: (seven_days.cache_write as f64 / 7.0).round() as i64,
    };

    let mut session_rows: Vec<SessionTotals> = sessions
        .into_values()
        .map(|(mut item, counts)| {
            item.finish(&counts);
            item
        })
        .collect();
    session_rows.sort_by_key(|item| Reverse(item.total));

    let mut model_rows: Vec<NamedTotals> = models
        .into_values()
        .map(|(mut item, counts)| {
            item.finish(&counts);
            item
        })
        .collect();
    model_rows.sort_by_key(|item| Reverse(item.total));

    let mut project_rows: Vec<NamedTotals> = projects
        .into_values()
        .map(|(mut item, counts)| {
            item.finish(&counts);
            item
        })
        .collect();
    project_rows.sort_by_key(|item| Reverse(item.total));

    UsageSnapshot {
        db_path: path.to_string_lossy().to_string(),
        message_count,
        today: totals_from(&today),
        yesterday: totals_from(&yesterday),
        seven_day_average: totals_from(&average),
        all_time: totals_from(&all_time),
        per_day: days
            .into_iter()
            .map(|(day, counts)| DayTotals::new(day, &counts))
            .collect(),
        sessions: session_rows,
        models: model_rows,
        projects: project_rows,
    }
}

pub fn get_usage(path: &Path) -> Result<UsageSnapshot, String> {
    let rows = read_rows(path)?;
    Ok(aggregate(rows, path, Local::now()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (PathBuf, Vec<UsageRow>) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("otd-rust-{}-{nanos}.db", std::process::id()));
        let _ = fs::remove_file(&path);
        let db = Connection::open(&path).unwrap();
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
        let message = |input: i64, output: i64, cache: i64, model: &str| {
            format!(
                "{{\"model\":{{\"id\":\"{model}\",\"providerID\":\"test\",\"variant\":\"\"}},\"tokens\":{{\"input\":{input},\"output\":{output},\"reasoning\":0,\"cache\":{{\"read\":{cache},\"write\":0}}}}}}"
            )
        };
        let now = Local::now();
        let today_ms = now
            .date_naive()
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        let yesterday_ms = (now.date_naive() - Duration::days(1))
            .and_hms_opt(10, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();
        db.execute(
            "INSERT INTO session_message VALUES ('m1','s1','assistant',?1,?2)",
            rusqlite::params![today_ms, message(10, 5, 100, "gpt-test")],
        )
        .unwrap();
        db.execute(
            "INSERT INTO session_message VALUES ('m2','s1','assistant',?1,?2)",
            rusqlite::params![yesterday_ms, message(20, 6, 200, "other-model")],
        )
        .unwrap();
        db.execute(
            "INSERT INTO session_message VALUES ('m3','s1','user',?1,'{}')",
            rusqlite::params![today_ms],
        )
        .unwrap();
        drop(db);
        let rows = read_rows(&path).unwrap();
        (path, rows)
    }

    #[test]
    fn reads_only_assistant_rows() {
        let (path, rows) = fixture();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].counts.input, 20);
        let _ = fs::remove_file(path);
    }

    fn session(label: &str, model: &str, project: &str) -> SessionTotals {
        SessionTotals {
            key: label.to_string(),
            label: label.to_string(),
            model: model.to_string(),
            project: project.to_string(),
            updated_at: 0,
            messages: 1,
            input: 0,
            output: 0,
            reasoning: 0,
            cache_read: 0,
            cache_write: 0,
            fresh: 0,
            total: 0,
        }
    }

    #[test]
    fn filter_sessions_combines_text_model_and_project() {
        let rows = vec![
            session("Alpha chat", "provider/a", "/work/one"),
            session("Beta chat", "provider/b", "/work/two"),
        ];
        assert_eq!(filter_sessions(&rows, "", "", "").len(), 2);
        assert_eq!(filter_sessions(&rows, "alpha", "", "").len(), 1);
        assert_eq!(filter_sessions(&rows, "", "provider/b", "").len(), 1);
        assert_eq!(filter_sessions(&rows, "", "", "/work/one").len(), 1);
        assert_eq!(
            filter_sessions(&rows, "chat", "provider/a", "/work/one").len(),
            1
        );
        assert!(filter_sessions(&rows, "", "provider/a", "/work/two").is_empty());
    }

    #[test]
    fn fingerprint_changes_when_database_changes() {
        let (path, _rows) = fixture();
        let before = db_fingerprint(&path);
        let db = Connection::open(&path).unwrap();
        db.execute(
            "INSERT INTO session_message VALUES ('m4','s1','assistant',0,'{}')",
            [],
        )
        .unwrap();
        drop(db);
        let after = db_fingerprint(&path);
        assert_ne!(before, after);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn database_is_read_only() {
        let (path, _rows) = fixture();
        let db = open_db(&path).unwrap();
        assert!(db
            .execute("INSERT INTO project VALUES ('x','x','x')", [])
            .is_err());
        drop(db);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn aggregates_totals_and_groups() {
        let (path, rows) = fixture();
        let snap = aggregate(rows, &path, Local::now());
        assert_eq!(snap.all_time.input, 30);
        assert_eq!(snap.all_time.output, 11);
        assert_eq!(snap.all_time.fresh, 41);
        assert_eq!(snap.all_time.cache_read, 300);
        assert_eq!(snap.all_time.total, 341);
        assert_eq!(snap.today.total, 115);
        assert_eq!(snap.yesterday.total, 226);
        assert_eq!(snap.seven_day_average.cache_read, 29);
        assert_eq!(snap.per_day.len(), 2);
        assert_eq!(snap.sessions.len(), 1);
        assert_eq!(snap.models.len(), 2);
        assert_eq!(snap.projects.len(), 1);
        let _ = fs::remove_file(path);
    }
}
