use crate::format::format_compact;
use crate::usage::DayTotals;
use std::collections::BTreeSet;

fn nice_step(value: f64) -> f64 {
    if value <= 0.0 {
        return 1.0;
    }
    let exponent = value.log10().floor();
    let base = 10f64.powf(exponent);
    let fraction = value / base;
    let multiplier = if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 2.5 {
        2.5
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };
    multiplier * base
}

pub fn area_chart_svg(days: &[DayTotals], width: f64, label_count: usize) -> String {
    if days.is_empty() {
        return String::new();
    }

    let height = 280.0;
    let left = if width < 600.0 { 40.0 } else { 52.0 };
    let right = 8.0;
    let top = 8.0;
    let bottom = 28.0;
    let plot_width = width - left - right;
    let plot_height = height - top - bottom;
    let baseline = top + plot_height;

    let max_total = days
        .iter()
        .map(|d| d.total.max(0) as f64)
        .fold(0.0, f64::max);
    let step = nice_step(max_total / 4.0);
    let y_max = step * (max_total / step).ceil().max(1.0);

    let n = days.len();
    let x = |i: usize| -> f64 {
        if n <= 1 {
            left
        } else {
            left + (i as f64 / (n - 1) as f64) * plot_width
        }
    };
    let y = |value: f64| -> f64 { baseline - (value / y_max).clamp(0.0, 1.0) * plot_height };

    let mut fresh = format!("M {:.1} {:.1}", x(0), baseline);
    for (i, day) in days.iter().enumerate() {
        fresh.push_str(&format!(" L {:.1} {:.1}", x(i), y(day.fresh as f64)));
    }
    fresh.push_str(&format!(" L {:.1} {:.1} Z", x(n - 1), baseline));

    let mut cache = format!("M {:.1} {:.1}", x(0), y(days[0].fresh as f64));
    for (i, day) in days.iter().enumerate().skip(1) {
        cache.push_str(&format!(" L {:.1} {:.1}", x(i), y(day.fresh as f64)));
    }
    for (i, day) in days.iter().enumerate().rev() {
        let stacked = (day.fresh + day.cache_read + day.cache_write) as f64;
        cache.push_str(&format!(" L {:.1} {:.1}", x(i), y(stacked)));
    }
    cache.push_str(" Z");

    let mut svg = format!(
        "<svg viewBox=\"0 0 {:.0} {:.0}\" style=\"width:100%;height:280px\" role=\"img\" aria-label=\"Stacked daily token usage, fresh tokens and cache reads\" preserveAspectRatio=\"none\">",
        width, height
    );

    let mut tick = 0.0;
    let mut guard = 0;
    while tick <= y_max + 0.001 && guard < 20 {
        let line_y = y(tick);
        svg.push_str(&format!(
            "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"var(--cv-line)\" stroke-width=\"1\" />",
            left, line_y, width - right, line_y
        ));
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"var(--cv-text-50)\" font-size=\"12\" text-anchor=\"start\">{}</text>",
            left - 8.0,
            line_y + 4.0,
            format_compact(tick.round() as i64)
        ));
        tick += step;
        guard += 1;
    }

    svg.push_str(&format!(
        "<path d=\"{}\" fill=\"var(--cv-text-50)\" fill-opacity=\"0.5\" stroke=\"var(--cv-text-75)\" stroke-width=\"1.5\" />",
        fresh
    ));
    svg.push_str(&format!(
        "<path d=\"{}\" fill=\"var(--cv-raised)\" fill-opacity=\"0.9\" stroke=\"var(--cv-text-25)\" stroke-width=\"1\" />",
        cache
    ));

    let mut labels = BTreeSet::new();
    labels.insert(0);
    labels.insert(n - 1);
    for i in 1..label_count {
        labels.insert(((n - 1) * i / label_count).min(n - 1));
    }
    for i in labels {
        let anchor = if i == 0 {
            "start"
        } else if i == n - 1 {
            "end"
        } else {
            "middle"
        };
        let label_x = if i == 0 {
            left
        } else if i == n - 1 {
            width - right
        } else {
            x(i)
        };
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.0}\" fill=\"var(--cv-text-50)\" font-size=\"12\" text-anchor=\"{}\">{}</text>",
            label_x,
            height - 6.0,
            anchor,
            days[i].day
        ));
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage::DayTotals;

    fn day(day: &str, fresh: i64, cache: i64) -> DayTotals {
        DayTotals {
            day: day.to_string(),
            input: fresh,
            output: 0,
            reasoning: 0,
            cache_read: cache,
            cache_write: 0,
            fresh,
            total: fresh + cache,
        }
    }

    #[test]
    fn draws_an_svg_with_both_series() {
        let svg = area_chart_svg(
            &[day("2026-11-17", 10, 100), day("2026-11-18", 20, 200)],
            1000.0,
            6,
        );
        assert!(svg.contains("<svg"));
        assert!(svg.contains("var(--cv-text-50)"));
        assert!(svg.contains("var(--cv-raised)"));
        assert!(svg.contains("2026-11-18"));
    }

    #[test]
    fn empty_input_returns_empty_string() {
        assert_eq!(area_chart_svg(&[], 1000.0, 6), "");
    }
}
