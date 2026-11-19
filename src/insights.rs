use serde::Serialize;

use crate::usage::UsageSnapshot;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Insights {
    pub spike_ratio: f64,
    pub busiest_day: Option<String>,
    pub busiest_day_total: i64,
    pub cache_ratio: f64,
    pub top_model: Option<String>,
    pub top_model_share: f64,
}

pub fn compute(snapshot: &UsageSnapshot) -> Insights {
    let spike_ratio = if snapshot.seven_day_average.total > 0 {
        snapshot.today.total as f64 / snapshot.seven_day_average.total as f64
    } else {
        0.0
    };

    let busiest = snapshot.per_day.iter().max_by_key(|day| day.total);
    let busiest_day = busiest.map(|day| day.day.clone());
    let busiest_day_total = busiest.map(|day| day.total).unwrap_or(0);

    let cache_ratio = if snapshot.all_time.total > 0 {
        snapshot.all_time.cache_read as f64 / snapshot.all_time.total as f64
    } else {
        0.0
    };

    let top_model = snapshot.models.first().map(|model| model.label.clone());
    let top_model_share = if snapshot.all_time.total > 0 {
        snapshot
            .models
            .first()
            .map(|model| model.total as f64 / snapshot.all_time.total as f64)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    Insights {
        spike_ratio,
        busiest_day,
        busiest_day_total,
        cache_ratio,
        top_model,
        top_model_share,
    }
}

pub fn spike_label(ratio: f64) -> &'static str {
    if ratio >= 2.0 {
        "well above the recent daily average"
    } else if ratio >= 1.2 {
        "above the recent daily average"
    } else if ratio > 0.0 && ratio <= 0.5 {
        "well below the recent daily average"
    } else {
        "in line with the recent daily average"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_spike_levels() {
        assert_eq!(spike_label(2.4), "well above the recent daily average");
        assert_eq!(spike_label(1.3), "above the recent daily average");
        assert_eq!(spike_label(1.0), "in line with the recent daily average");
        assert_eq!(spike_label(0.3), "well below the recent daily average");
    }
}
