use super::{Config, Metric};
use serde::{Deserialize, Serialize};

/// An optional partition of a numeric KPI by a text column in the annex.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Breakdown {
    /// Exact property-column heading, ignoring surrounding whitespace.
    pub column: String,
    /// Nonempty list of allowed classes, in display order.
    pub classes: Vec<MetricClass>,
}

/// One allowed value of a KPI's property column.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricClass {
    /// Portable identifier unique within the KPI.
    pub id: String,
    /// Display name, used unchanged in every interface and output language.
    pub label: String,
    /// Exact Excel text to match, case-sensitive after trimming whitespace.
    pub value: String,
}

/// A result column: a KPI total or one of its configured classes.
///
/// All GUI tables, charts, and exports use the same ordered columns.
pub struct Indicator<'a> {
    /// KPI ID for a total, or `kpi_id.class_id` for a class. Dots cannot occur
    /// inside configured IDs, so these result identifiers are unambiguous.
    pub id: String,
    /// Numeric source, aggregation, missing-value policy, unit, and precision.
    pub metric: &'a Metric,
    /// Class definition, or `None` for the KPI total.
    pub class: Option<&'a MetricClass>,
}

impl Indicator<'_> {
    /// Display a total as the KPI label, or a class as `KPI · Class`.
    pub fn label(&self) -> String {
        let label = &self.metric.label;
        match self.class {
            Some(class) => format!("{label} · {}", class.label),
            None => label.clone(),
        }
    }
}

impl Config {
    /// Ordered result columns: each KPI total followed by its configured classes.
    /// This order defines every numeric vector in [`crate::analysis::Analysis`].
    pub fn indicators(&self) -> Vec<Indicator<'_>> {
        self.metrics
            .iter()
            .flat_map(|metric| {
                std::iter::once(Indicator {
                    id: metric.id.clone(),
                    metric,
                    class: None,
                })
                .chain(
                    metric
                        .breakdown
                        .iter()
                        .flat_map(|breakdown| &breakdown.classes)
                        .map(|class| Indicator {
                            id: format!("{}.{}", metric.id, class.id),
                            metric,
                            class: Some(class),
                        }),
                )
            })
            .collect()
    }
}
