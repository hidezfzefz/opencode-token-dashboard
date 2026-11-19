use std::io::Write;
use std::path::{Path, PathBuf};

macro_rules! out {
    ($fmt:expr) => {
        let _ = writeln!(std::io::stdout(), $fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        let _ = writeln!(std::io::stdout(), $fmt, $($arg)*);
    };
}

use crate::csv;
use crate::format::{format_compact, format_number};
use crate::insights;
use crate::usage;

fn parse_db_path(args: &[String]) -> PathBuf {
    let mut path = usage::default_db_path();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--db-path" && index + 1 < args.len() {
            path = PathBuf::from(&args[index + 1]);
            index += 2;
        } else {
            index += 1;
        }
    }
    path
}

fn parse_out_dir(args: &[String]) -> PathBuf {
    let mut out = PathBuf::from(".");
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--out" && index + 1 < args.len() {
            out = PathBuf::from(&args[index + 1]);
            index += 2;
        } else {
            index += 1;
        }
    }
    out
}

fn load(path: &Path) -> Result<usage::UsageSnapshot, String> {
    usage::get_usage(path)
}

pub fn report(args: &[String]) -> i32 {
    let path = parse_db_path(args);
    let snapshot = match load(&path) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            eprintln!("Could not read {}: {error}", path.display());
            return 1;
        }
    };
    let insights = insights::compute(&snapshot);
    out!("Token usage — {}", snapshot.db_path);
    out!(
        "Today:     {} tokens (fresh {})",
        format_compact(snapshot.today.total),
        format_compact(snapshot.today.fresh)
    );
    out!(
        "All time:  {} tokens (fresh {})",
        format_compact(snapshot.all_time.total),
        format_compact(snapshot.all_time.fresh)
    );
    out!("Yesterday: {}", format_compact(snapshot.yesterday.total));
    out!(
        "7-day daily average: {} — today is {:.1}× that ({})",
        format_compact(snapshot.seven_day_average.total),
        insights.spike_ratio,
        insights::spike_label(insights.spike_ratio)
    );
    out!(
        "Busiest day: {}",
        insights
            .busiest_day
            .as_ref()
            .map(|day| format!("{day} ({})", format_compact(insights.busiest_day_total)))
            .unwrap_or_else(|| "none".to_string())
    );
    out!(
        "Top model: {}",
        insights.top_model.as_deref().unwrap_or("none")
    );
    out!(
        "Assistant messages: {} across {} days",
        format_number(snapshot.message_count as i64),
        snapshot.per_day.len()
    );
    0
}

pub fn export(args: &[String]) -> i32 {
    let path = parse_db_path(args);
    let out = parse_out_dir(args);
    let snapshot = match load(&path) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            eprintln!("Could not read {}: {error}", path.display());
            return 1;
        }
    };
    if let Err(error) = std::fs::create_dir_all(&out) {
        eprintln!("Could not create {}: {error}", out.display());
        return 1;
    }
    let sessions = out.join("opencode-token-usage-sessions.csv");
    let daily = out.join("opencode-token-usage-daily.csv");
    let mut days = snapshot.per_day.clone();
    days.reverse();
    if let Err(error) = std::fs::write(&sessions, csv::sessions_to_csv(&snapshot.sessions)) {
        eprintln!("Could not write {}: {error}", sessions.display());
        return 1;
    }
    if let Err(error) = std::fs::write(&daily, csv::daily_to_csv(&days)) {
        eprintln!("Could not write {}: {error}", daily.display());
        return 1;
    }
    out!(
        "Wrote {} ({} rows)",
        sessions.display(),
        snapshot.sessions.len()
    );
    out!("Wrote {} ({} rows)", daily.display(), days.len());
    0
}
