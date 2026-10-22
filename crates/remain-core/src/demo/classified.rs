use super::configuration::localized;
use crate::config::{Breakdown, Config, MetricClass, Project};
use anyhow::Result;
use rust_xlsxwriter::{Format, Worksheet};

pub(super) fn configure(config: &mut Config, projects: &mut [Project]) {
    let text = |en, de, fr| localized(&config.locale, en, de, fr);
    let class = |id: &str, en, de, fr| {
        let label = text(en, de, fr);
        MetricClass {
            id: id.into(),
            value: label.clone(),
            label,
        }
    };
    config.metrics[0].breakdown = Some(Breakdown {
        column: text("Publication type", "Publikationstyp", "Type de publication"),
        classes: vec![
            class("conference", "Conference", "Konferenz", "Conférence"),
            class("journal", "Journal", "Zeitschrift", "Revue"),
            class("book", "Book", "Buch", "Livre"),
        ],
    });
    config.metrics[2].breakdown = Some(Breakdown {
        column: text("Assessment phase", "Bewertungsphase", "Phase d’évaluation"),
        classes: vec![
            class("baseline", "Baseline", "Ausgangswert", "État initial"),
            class("followup", "Follow-up", "Nachverfolgung", "Suivi"),
        ],
    });
    // This also leaves one configured group empty, without inventing observations.
    projects[0].group = None;
}

pub(super) fn properties(
    sheet: &mut Worksheet,
    config: &Config,
    project: usize,
    header: &Format,
) -> Result<()> {
    let rows: [&[(usize, Option<usize>)]; 3] = [
        &[(0, Some(0)), (1, Some(1))],
        &[(1, Some(0))],
        &[(0, Some(1)), (0, Some(0)), (1, None)],
    ];
    for (offset, metric) in [0, 2].into_iter().enumerate() {
        let breakdown = config.metrics[metric]
            .breakdown
            .as_ref()
            .expect("demo classes");
        let column = config.metrics.len() as u16 + offset as u16;
        sheet.write_string_with_format(0, column, &breakdown.column, header)?;
        sheet.set_column_width(column, 26)?;
        for (row, &(publication, phase)) in rows[project].iter().enumerate() {
            let selected = if metric == 0 {
                Some(publication)
            } else {
                phase
            };
            if let Some(index) = selected {
                sheet.write_string(row as u32 + 1, column, &breakdown.classes[index].value)?;
            }
        }
    }
    Ok(())
}
