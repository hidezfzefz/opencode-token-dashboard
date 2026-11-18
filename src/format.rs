pub fn format_number(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if negative {
        out.push('-');
    }
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn compact_unit(value: f64, suffix: &str) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    format!("{}{}", rounded, suffix)
}

pub fn format_compact(value: i64) -> String {
    let abs = value.unsigned_abs() as f64;
    if abs >= 1_000_000_000.0 {
        compact_unit(value as f64 / 1_000_000_000.0, "B")
    } else if abs >= 1_000_000.0 {
        compact_unit(value as f64 / 1_000_000.0, "M")
    } else if abs >= 1_000.0 {
        compact_unit(value as f64 / 1_000.0, "K")
    } else {
        value.to_string()
    }
}

pub fn format_percent(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "0%".to_string();
    }
    format!("{}%", ((part as f64 / whole as f64) * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_thousands() {
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(-1234), "-1,234");
    }

    #[test]
    fn compact_notation() {
        assert_eq!(format_compact(1_714_808_757), "1.7B");
        assert_eq!(format_compact(34_800_000), "34.8M");
        assert_eq!(format_compact(842), "842");
        assert_eq!(format_compact(0), "0");
    }

    #[test]
    fn percentages() {
        assert_eq!(format_percent(1, 3), "33%");
        assert_eq!(format_percent(5, 0), "0%");
    }
}
