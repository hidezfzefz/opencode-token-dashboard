use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::usage::UsageSnapshot;

/// Prices are USD per one million tokens. The user supplies these; no prices
/// are invented or fetched from a network.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Rates {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pricing {
    pub models: HashMap<String, Rates>,
}

#[derive(Clone, Debug)]
pub struct ModelCost {
    pub model: String,
    pub messages: i64,
    pub total: f64,
}

#[derive(Clone, Debug)]
pub struct CostSummary {
    pub rows: Vec<ModelCost>,
    pub total: f64,
    pub configured: bool,
}

/// Resolve a model key against the pricing file. Accepts the full
/// `provider/model#variant`, the key without variant, or a bare model id.
pub fn resolve_rates<'a>(pricing: &'a Pricing, model: &str) -> Option<&'a Rates> {
    if let Some(rates) = pricing.models.get(model) {
        return Some(rates);
    }
    let without_variant = model.split('#').next().unwrap_or(model);
    if let Some(rates) = pricing.models.get(without_variant) {
        return Some(rates);
    }
    let bare = without_variant
        .rsplit('/')
        .next()
        .unwrap_or(without_variant);
    if let Some(rates) = pricing.models.get(bare) {
        return Some(rates);
    }
    pricing
        .models
        .iter()
        .find(|(key, _)| {
            key.split('#')
                .next()
                .and_then(|part| part.rsplit('/').next())
                == Some(bare)
        })
        .map(|(_, rates)| rates)
}

fn cost_of(tokens: i64, rate_per_million: f64) -> f64 {
    tokens as f64 * rate_per_million / 1_000_000.0
}

pub fn estimate(snapshot: &UsageSnapshot, pricing: &Pricing) -> CostSummary {
    let mut rows = Vec::new();
    let mut total = 0.0;
    for model in &snapshot.models {
        let Some(rates) = resolve_rates(pricing, &model.key) else {
            continue;
        };
        let cost = cost_of(model.input, rates.input)
            + cost_of(model.output, rates.output)
            + cost_of(model.cache_read, rates.cache_read)
            + cost_of(model.cache_write, rates.cache_write);
        total += cost;
        rows.push(ModelCost {
            model: model.label.clone(),
            messages: model.messages,
            total: cost,
        });
    }
    CostSummary {
        rows,
        total,
        configured: !pricing.models.is_empty(),
    }
}

pub fn format_money(value: f64) -> String {
    if value >= 1_000.0 {
        format!("${}", crate::format::format_number(value.round() as i64))
    } else if value >= 1.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pricing() -> Pricing {
        let mut models = HashMap::new();
        models.insert(
            "test/gpt".to_string(),
            Rates {
                input: 2.0,
                output: 8.0,
                cache_read: 0.5,
                cache_write: 2.5,
            },
        );
        Pricing { models }
    }

    #[test]
    fn resolves_full_bare_and_variant_keys() {
        let pricing = pricing();
        assert!(resolve_rates(&pricing, "test/gpt").is_some());
        assert!(resolve_rates(&pricing, "test/gpt#max").is_some());
        assert!(resolve_rates(&pricing, "gpt").is_some());
        assert!(resolve_rates(&pricing, "unknown").is_none());
    }

    #[test]
    fn formats_money() {
        assert_eq!(format_money(1234.5), "$1,235");
        assert_eq!(format_money(12.345), "$12.35");
        assert_eq!(format_money(0.0123), "$0.0123");
    }
}
