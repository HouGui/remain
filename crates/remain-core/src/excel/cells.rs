use crate::config::{Breakdown, Metric, Missing};
use anyhow::{Result, bail};
use calamine::Data;

pub(super) fn blank(value: Option<&Data>) -> bool {
    matches!(value, None | Some(Data::Empty))
        || matches!(value, Some(Data::String(text)) if text.trim().is_empty())
}

pub(super) fn numeric(cell: Option<&Data>, metric: &Metric, row: u32) -> Result<Option<f64>> {
    match cell {
        Some(Data::Float(value)) if value.is_finite() => Ok(Some(*value)),
        Some(Data::Int(value)) if value.unsigned_abs() <= (1_u64 << 53) => Ok(Some(*value as f64)),
        value if blank(value) => match metric.missing {
            Missing::Zero => Ok(Some(0.0)),
            Missing::Skip => Ok(None),
            Missing::Error => bail!(
                "Row {}, column {:?}: missing numeric value",
                row + 1,
                metric.column
            ),
        },
        _ => bail!(
            "Row {}, column {:?}: expected a finite numeric cell (not text, dates, booleans, or Excel errors)",
            row + 1,
            metric.column
        ),
    }
}

pub(super) fn class(
    cell: Option<&Data>,
    value: Option<f64>,
    breakdown: &Breakdown,
    row: u32,
) -> Result<Option<usize>> {
    if blank(cell) {
        if value.is_none() {
            return Ok(None);
        }
        bail!(
            "Row {}, column {:?}: a class is required for a numeric KPI value (including zero)",
            row + 1,
            breakdown.column
        );
    }
    let Some(Data::String(text)) = cell else {
        bail!(
            "Row {}, column {:?}: expected a text class value",
            row + 1,
            breakdown.column
        );
    };
    match breakdown
        .classes
        .iter()
        .position(|class| class.value.trim() == text.trim())
    {
        Some(index) => Ok(Some(index)),
        None => bail!(
            "Row {}, column {:?}: unknown class value {text:?}; use a configured class Excel value",
            row + 1,
            breakdown.column
        ),
    }
}
