use crate::{
    analysis::Analysis,
    config::{Aggregation, Indicator},
    locale,
};
use std::{collections::HashMap, fmt::Write};

pub(crate) fn render(workspace: &crate::Workspace, analysis: &Analysis) -> String {
    let config = workspace.config();
    let indicators = config.indicators();
    let t = |key| escape(locale::text(&config.locale, &config.translations, key));
    let title = escape(config.title.as_str());
    let projects: HashMap<_, _> = workspace
        .projects()
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    let groups: HashMap<_, _> = config.groups.iter().map(|g| (g.id.as_str(), g)).collect();
    let categories: HashMap<_, _> = config
        .categories
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();
    let mut out = format!(
        r#"<!doctype html><html lang="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><style>{}</style></head><body><main><header><span class="brand">REMAIN</span><h1>{title}</h1><p>{}: {}</p></header><nav><a href="report.pdf">{}</a>"#,
        escape(&config.locale),
        include_str!("report.css"),
        t("period"),
        escape(&config.period),
        t("pdf")
    );
    if !indicators.is_empty() {
        let _ = write!(out, "<a href=\"annex.xlsx\">{}</a>", t("excel"));
    }
    out.push_str("</nav>");
    if !analysis.is_complete() {
        let _ = write!(
            out,
            "<aside class=warning><h2>{}</h2><p>{}</p></aside>",
            t("incomplete_report"),
            t("partial_explanation")
        );
    }
    let _ = write!(
        out,
        "<section><h2>{}</h2><p>{}: {} / {}</p>",
        t("submissions"),
        t("included_projects"),
        analysis.projects.len(),
        workspace.projects().len()
    );
    if !indicators.is_empty() {
        let _ = write!(
            out,
            "<p>{}: {} / {}</p>",
            t("kpi_submissions"),
            analysis.projects.iter().filter(|p| p.has_kpis).count(),
            workspace.projects().len()
        );
    }
    out.push_str("<ul>");
    for submission in &analysis.submissions {
        let _ = write!(
            out,
            "<li><strong>{} ({})</strong>: {}",
            escape(projects[submission.project.as_str()].label.as_str()),
            escape(&submission.project),
            if submission.is_complete() {
                t("submitted")
            } else {
                format!(
                    "{}: {}",
                    t("missing_submissions"),
                    escape(&submission.missing.join(", "))
                )
            }
        );
        if !submission.optional_missing.is_empty() {
            let _ = write!(
                out,
                "<small>{}: {}</small>",
                t("optional_missing"),
                escape(&submission.optional_missing.join(", "))
            );
        }
        out.push_str("</li>");
    }
    out.push_str("</ul></section>");
    if !indicators.is_empty() {
        let _ = write!(out, "<section><h2>{}</h2><div class=cards>", t("total"));
        for (indicator, value) in indicators.iter().zip(&analysis.overall) {
            let metric = indicator.metric;
            let operation = match metric.aggregation {
                Aggregation::Sum => "sum",
                Aggregation::Mean => "mean",
                Aggregation::Min => "min",
                Aggregation::Max => "max",
                Aggregation::Count => "count",
            };
            let _ = write!(
                out,
                "<article><span>{}</span><strong>{}</strong><small>{} · {}</small></article>",
                escape(&indicator.label()),
                locale::number(*value, metric.decimals, &config.locale),
                t(operation),
                escape(&metric.unit)
            );
        }
        out.push_str("</div></section>");
    }
    out.push_str("<section>");
    for (plural, singular, definitions, results) in [
        ("groups", "group", &groups, &analysis.groups),
        ("categories", "category", &categories, &analysis.categories),
    ] {
        let _ = write!(
            out,
            "<h2>{}</h2><div class=scroll><table><thead><tr><th>{}</th><th>{}</th>",
            t(plural),
            t(singular),
            t("projects")
        );
        headers(&mut out, &indicators);
        out.push_str("</tr></thead><tbody>");
        for row in results {
            let definition = row.id.as_deref().and_then(|id| definitions.get(id));
            let name = definition
                .map(|c| escape(c.label.as_str()))
                .unwrap_or_else(|| t("unassigned"));
            let name = match definition {
                Some(item) if row.expected_projects > 0 => {
                    format!("<a href=\"{plural}/{}.pdf\">{name}</a>", item.id)
                }
                _ => name,
            };
            let _ = write!(
                out,
                "<tr><th>{name}</th><td>{} / {}</td>",
                row.projects, row.expected_projects
            );
            cells(&mut out, &indicators, &config.locale, &row.values);
            out.push_str("</tr>");
        }
        out.push_str("</tbody></table></div></section><section>");
    }
    let _ = write!(
        out,
        "<h2>{}</h2><div class=scroll><table><thead><tr><th>{}</th><th>{}</th><th>{}</th>",
        t("projects"),
        t("project"),
        t("group"),
        t("category")
    );
    headers(&mut out, &indicators);
    out.push_str("</tr></thead><tbody>");
    for row in &analysis.projects {
        let project = projects.get(row.id.as_str()).expect("analyzed project");
        let group = row.group.as_deref().and_then(|id| groups.get(id));
        let category = row.category.as_deref().and_then(|id| categories.get(id));
        let _ = write!(
            out,
            "<tr><th><a href=\"projects/{}.pdf\">{}</a><small>{}</small></th><td>{}</td><td>{}</td>",
            row.id,
            escape(project.label.as_str()),
            escape(&row.id),
            group
                .map(|g| escape(g.label.as_str()))
                .unwrap_or_else(|| t("unassigned")),
            category
                .map(|c| escape(c.label.as_str()))
                .unwrap_or_else(|| t("unassigned"))
        );
        cells(&mut out, &indicators, &config.locale, &row.values);
        out.push_str("</tr>");
    }
    let _ = write!(
        out,
        "</tbody></table></div></section><footer>{}<br>REMAIN · {} {} · {} {}</footer></main></body></html>",
        t("missing"),
        analysis.projects.len(),
        t("projects"),
        analysis.observations.len(),
        t("rows")
    );
    out
}

fn headers(out: &mut String, indicators: &[Indicator<'_>]) {
    for indicator in indicators {
        let metric = indicator.metric;
        let _ = write!(
            out,
            "<th>{}<small>{}</small></th>",
            escape(&indicator.label()),
            escape(&metric.unit)
        );
    }
}

fn cells(out: &mut String, indicators: &[Indicator<'_>], locale: &str, values: &[Option<f64>]) {
    for (indicator, value) in indicators.iter().zip(values) {
        let metric = indicator.metric;
        let _ = write!(
            out,
            "<td>{}</td>",
            locale::number(*value, metric.decimals, locale)
        );
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
