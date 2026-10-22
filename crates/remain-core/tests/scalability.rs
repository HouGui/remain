//! Explicit disk-backed scalability check, also run by Linux CI.
mod common;
use anyhow::{Result, ensure};
use calamine::{Reader, Xlsx, open_workbook};
use lopdf::Document;
use remain_core::{Workspace, config::Classification};
use std::{fs, time::Instant};

#[test]
#[ignore = "Creates and exports 1000 real project submissions; run explicitly"]
fn large_portfolio_preserves_all_projects_pages_and_observations() -> Result<()> {
    let count: usize = std::env::var("REMAIN_SCALE_PROJECTS")
        .unwrap_or_else(|_| "1000".into())
        .parse()?;
    ensure!((100..=100_000).contains(&count), "Use 100–100000 projects");
    let (tmp, template) = common::classified_workspace("en")?;
    let root = tmp.path().join("large");
    fs::create_dir(&root)?;
    let mut config = template.config().clone();
    config.groups = (0..20)
        .map(|i| Classification {
            id: format!("g{i}"),
            label: format!("Group {i}"),
        })
        .collect();
    config.categories = (0..10)
        .map(|i| Classification {
            id: format!("c{i}"),
            label: format!("Category {i}"),
        })
        .collect();
    let mut projects = Vec::new();
    for i in 0..count {
        let mut project = template.projects()[0].clone();
        project.id = format!("p{i}");
        project.label = format!("Project {i}");
        project.group = (i % 7 != 0).then(|| format!("g{}", i % 20));
        project.category = (i % 5 != 0).then(|| format!("c{}", i % 10));
        fs::create_dir(root.join(&project.id))?;
        fs::copy(
            template.root().join("atlas/annex.xlsx"),
            root.join(&project.id).join("annex.xlsx"),
        )?;
        fs::copy(
            template.root().join("atlas/part-1.pdf"),
            root.join(&project.id).join("part-1.pdf"),
        )?;
        projects.push(project);
    }
    let workspace = Workspace::with_projects(&root, config, projects)?;
    let started = Instant::now();
    let analysis = workspace.analyze()?;
    let analysis_time = started.elapsed();
    let started = Instant::now();
    let output = tmp.path().join("export");
    let exported = workspace.export(&output)?;
    let export_time = started.elapsed();
    assert_eq!(analysis.overall, exported.overall);
    assert_eq!(exported.projects.len(), count);
    assert_eq!(exported.observations.len(), count * 2);
    assert_eq!(exported.overall[0], Some(count as f64 * 5.));
    assert_eq!(exported.overall[1], Some(count as f64 * 3.));
    assert_eq!(exported.overall[2], Some(count as f64 * 2.));
    assert_eq!(exported.overall[3], None);
    assert_eq!(exported.overall[4], Some(count as f64));
    assert!((exported.overall[5].expect("mean") - 0.85).abs() < 1e-10);
    assert!((exported.overall[6].expect("baseline mean") - 0.8).abs() < 1e-10);
    assert!((exported.overall[7].expect("follow-up mean") - 0.9).abs() < 1e-10);
    for scopes in [&exported.groups, &exported.categories] {
        assert_eq!(scopes.iter().map(|s| s.projects).sum::<usize>(), count);
        assert_eq!(
            scopes.iter().filter_map(|s| s.values[0]).sum::<f64>(),
            count as f64 * 5.
        );
    }
    let pdf = Document::load(output.join("report.pdf"))?;
    let front_pages = pdf.get_pages().len() - count;
    assert!(front_pages > 1);
    assert!(
        pdf.extract_text(&[front_pages as u32 + 1])?
            .contains("ATLAS")
    );
    assert_eq!(fs::read_dir(output.join("projects"))?.count(), count);
    let mut workbook: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    assert_eq!(
        workbook.worksheet_range("Observations")?.height(),
        count * 2 + 1
    );
    println!(
        "{count} projects, {} observations: analyze {analysis_time:?}; export {export_time:?}",
        count * 2
    );
    Ok(())
}
