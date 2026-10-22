use super::*;
use crate::config::{Annex, Metric, Missing};
use std::collections::BTreeMap;

fn calculate(method: Aggregation, input: &[Option<f64>]) -> Result<Option<f64>> {
    let config = Config {
        schema_version: crate::config::SCHEMA_VERSION,
        title: "Test".into(),
        locale: "en".into(),
        period: "2026".into(),
        annex: Some(Annex {
            sheet: "Data".into(),
            header_row: 1,
        }),
        contents_font: None,
        inputs: Default::default(),
        groups: vec![],
        categories: vec![],
        projects: "projects.csv".into(),
        metrics: vec![Metric {
            id: "value".into(),
            label: "Value".into(),
            column: "Value".into(),
            aggregation: method,
            missing: Missing::Skip,
            unit: String::new(),
            decimals: 2,
            breakdown: None,
        }],
        translations: BTreeMap::new(),
    };
    let rows: Vec<_> = input
        .iter()
        .enumerate()
        .map(|(i, value)| Observation {
            project: "p".into(),
            row: i as u32 + 2,
            values: vec![*value],
        })
        .collect();
    Ok(summarize(&config, &rows.iter().collect::<Vec<_>>())?[0])
}

#[test]
fn all_aggregations_use_present_values() -> Result<()> {
    let input = [Some(2.), None, Some(0.), Some(-1.), Some(7.)];
    for (method, expected) in [
        (Aggregation::Sum, 8.),
        (Aggregation::Mean, 2.),
        (Aggregation::Min, -1.),
        (Aggregation::Max, 7.),
        (Aggregation::Count, 4.),
    ] {
        assert_eq!(calculate(method, &input)?, Some(expected));
    }
    Ok(())
}

#[test]
fn missing_and_empty_are_not_zero_except_count() -> Result<()> {
    for input in [vec![], vec![None, None]] {
        for method in [
            Aggregation::Sum,
            Aggregation::Mean,
            Aggregation::Min,
            Aggregation::Max,
        ] {
            assert_eq!(calculate(method, &input)?, None);
        }
        assert_eq!(calculate(Aggregation::Count, &input)?, Some(0.));
    }
    Ok(())
}

#[test]
fn finite_inputs_cannot_produce_infinite_exports() {
    assert!(calculate(Aggregation::Sum, &[Some(f64::MAX), Some(f64::MAX)]).is_err());
}
