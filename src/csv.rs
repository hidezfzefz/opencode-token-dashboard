use crate::usage::{DayTotals, SessionTotals};

pub fn escape_csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn to_csv(header: &[&str], rows: &[Vec<String>]) -> String {
    let header_line = header
        .iter()
        .map(|cell| escape_csv_cell(cell))
        .collect::<Vec<_>>()
        .join(",");
    let mut lines = vec![header_line];
    for row in rows {
        lines.push(
            row.iter()
                .map(|cell| escape_csv_cell(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    lines.join("\n")
}

pub fn sessions_to_csv(rows: &[SessionTotals]) -> String {
    let header = [
        "Session",
        "Last active",
        "Model",
        "Project",
        "Messages",
        "Input",
        "Output",
        "Reasoning",
        "Cache read",
        "Cache write",
        "Fresh",
        "Total",
    ];
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            vec![
                row.label.clone(),
                chrono::DateTime::from_timestamp_millis(row.updated_at)
                    .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                    .unwrap_or_default(),
                row.model.clone(),
                row.project.clone(),
                row.messages.to_string(),
                row.input.to_string(),
                row.output.to_string(),
                row.reasoning.to_string(),
                row.cache_read.to_string(),
                row.cache_write.to_string(),
                row.fresh.to_string(),
                row.total.to_string(),
            ]
        })
        .collect();
    to_csv(&header, &body)
}

pub fn daily_to_csv(rows: &[DayTotals]) -> String {
    let header = [
        "Day",
        "Fresh",
        "Input",
        "Output",
        "Reasoning",
        "Cache read",
        "Cache write",
        "Total",
    ];
    let body: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            vec![
                row.day.clone(),
                row.fresh.to_string(),
                row.input.to_string(),
                row.output.to_string(),
                row.reasoning.to_string(),
                row.cache_read.to_string(),
                row.cache_write.to_string(),
                row.total.to_string(),
            ]
        })
        .collect();
    to_csv(&header, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_cells() {
        assert_eq!(escape_csv_cell("hello"), "hello");
        assert_eq!(escape_csv_cell("a,b"), "\"a,b\"");
        assert_eq!(escape_csv_cell("a \"b\""), "\"a \"\"b\"\"\"");
    }

    #[test]
    fn writes_header_and_rows() {
        let csv = to_csv(&["A", "B"], &[vec!["1".to_string(), "x,y".to_string()]]);
        assert_eq!(csv, "A,B\n1,\"x,y\"");
    }
}
