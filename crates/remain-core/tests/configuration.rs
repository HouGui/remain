//! Configuration boundary and portability tests.
mod common;
use anyhow::Result;
use remain_core::{Workspace, config::Config};
use std::fs;

#[test]
fn rejects_unknown_fields_and_schema_versions() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let mut value = serde_json::to_value(workspace.config())?;
    value["unexpected"] = true.into();
    assert!(serde_json::from_value::<Config>(value).is_err());
    let mut config = workspace.config().clone();
    for version in [0, 1, 99] {
        config.schema_version = version;
        let error = config
            .validate()
            .expect_err("unsupported version")
            .to_string();
        assert!(error.contains("expected 2"), "{error}");
    }
    Ok(())
}

#[test]
fn duplicate_identifiers_and_columns_are_rejected() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for kind in 0..4 {
        let mut config = workspace.config().clone();
        let mut projects = workspace.projects().to_vec();
        match kind {
            0 => projects[1].id = projects[0].id.to_uppercase(),
            1 => config.groups[1].id = config.groups[0].id.clone(),
            2 => config.metrics[1].id = config.metrics[0].id.clone(),
            _ => config.metrics[1].column = format!(" {} ", config.metrics[0].column),
        }
        assert!(
            config
                .validate()
                .and_then(|()| config.validate_projects(&projects))
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn unknown_group_parent_and_parent_cycles_fail() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for kind in 0..4 {
        let config = workspace.config().clone();
        let mut projects = workspace.projects().to_vec();
        match kind {
            0 => projects[0].group = Some("unknown".into()),
            1 => projects[0].parent = Some("unknown".into()),
            2 => projects[0].parent = Some("atlas".into()),
            _ => {
                projects[0].parent = Some("nova".into());
                projects[1].parent = Some("atlas".into());
            }
        }
        assert!(
            config
                .validate()
                .and_then(|()| config.validate_projects(&projects))
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn unsafe_paths_reserved_ids_and_duplicate_files_fail() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for path in [
        "../outside.xlsx",
        "/tmp/annex.xlsx",
        "C:\\annex.xlsx",
        "./annex.xlsx",
        "atlas/./annex.xlsx",
        "atlas//annex.xlsx",
        "wrong.xls",
    ] {
        let mut config = workspace.config().clone();
        config.projects = path.into();
        assert!(config.validate().is_err(), "{path}");
    }
    for id in ["../id", "", "CON", "lpt9", "a/b"] {
        let config = workspace.config();
        let mut projects = workspace.projects().to_vec();
        projects[0].id = id.into();
        assert!(config.validate_projects(&projects).is_err(), "{id}");
    }
    let mut config = workspace.config().clone();
    config
        .inputs
        .documents
        .push(config.inputs.documents[0].clone());
    assert!(config.validate().is_err());
    Ok(())
}

#[test]
fn deep_parent_chains_validate_without_recursion_and_reject_cycles() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let config = workspace.config().clone();
    let prototype = workspace.projects()[0].clone();
    let mut projects: Vec<_> = (0..10_000)
        .map(|i| {
            let mut project = prototype.clone();
            project.id = format!("p{i}");
            project.parent = (i < 9_999).then(|| format!("p{}", i + 1));
            project
        })
        .collect();
    config.validate_projects(&projects)?;
    projects[9_999].parent = Some("p0".into());
    assert!(config.validate_projects(&projects).is_err());
    Ok(())
}

#[test]
fn arbitrary_language_labels_and_catalog_overrides_roundtrip() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.locale = "ja".into();
    config.title = "研究報告".into();
    config.translations.insert("open".into(), "開く".into());
    config.validate()?;
    let decoded: Config = serde_json::from_str(&serde_json::to_string(&config)?)?;
    assert_eq!(decoded.title, "研究報告");
    assert_eq!(decoded.translations["open"], "開く");
    Ok(())
}

#[test]
fn every_configured_name_requires_a_nonempty_plain_string() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    let original = serde_json::to_value(workspace.config())?;
    for path in [
        "/title",
        "/groups/0/label",
        "/categories/0/label",
        "/inputs/documents/0/label",
        "/metrics/0/label",
        "/metrics/0/breakdown/classes/0/label",
    ] {
        assert!(
            original.pointer(path).expect("configured name").is_string(),
            "{path}"
        );
        for text in ["Research / Forschung / Recherche", "研究報告", " Étude "] {
            let mut value = original.clone();
            *value.pointer_mut(path).expect("configured name") = text.into();
            let config: Config = serde_json::from_value(value.clone())?;
            config.validate()?;
            assert_eq!(serde_json::to_value(config)?, value, "{path}");
        }
        for text in ["", " \t\n"] {
            let mut value = original.clone();
            *value.pointer_mut(path).expect("configured name") = text.into();
            let config: Config = serde_json::from_value(value)?;
            assert!(config.validate().is_err(), "{path}");
        }
        for alternative in [
            serde_json::json!({"en": "Research", "de": "Forschung", "fr": "Recherche"}),
            serde_json::json!({}),
            serde_json::json!(["Research"]),
            serde_json::Value::Null,
        ] {
            let mut value = original.clone();
            *value.pointer_mut(path).expect("configured name") = alternative;
            assert!(serde_json::from_value::<Config>(value).is_err(), "{path}");
        }
    }
    Ok(())
}

#[test]
fn empty_required_fields_and_out_of_range_options_fail() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for case in 0..8 {
        let mut config = workspace.config().clone();
        match case {
            0 => config.projects.clear(),
            1 => config.inputs.directory = "../{id}".into(),
            2 => config.metrics.clear(),
            3 => config.inputs.documents.clear(),
            4 => config.metrics[0].decimals = 7,
            5 => config.annex.as_mut().expect("KPI settings").header_row = 0,
            6 => config.locale.clear(),
            _ => {
                config
                    .translations
                    .insert("not_a_key".into(), "value".into());
            }
        }
        assert!(config.validate().is_err());
    }
    Ok(())
}

#[test]
fn workspace_resolution_does_not_depend_on_working_directory() -> Result<()> {
    let (_tmp, workspace) = common::workspace("de")?;
    let loaded = Workspace::load(workspace.root().join("workspace.json"))?;
    assert_eq!(loaded.analyze()?.overall[0], Some(12.));
    fs::write(workspace.root().join("bad.json"), "{")?;
    assert!(Workspace::load(workspace.root().join("bad.json")).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_escape_or_duplicate_workspace_inputs() -> Result<()> {
    use std::os::unix::fs::symlink;
    let (tmp, workspace) = common::workspace("en")?;
    let outside = tmp.path().join("outside.xlsx");
    fs::copy(workspace.root().join("atlas/annex.xlsx"), &outside)?;
    fs::remove_file(workspace.root().join("atlas/annex.xlsx"))?;
    symlink(&outside, workspace.root().join("atlas/annex.xlsx"))?;
    assert!(workspace.analyze().is_err());
    fs::remove_file(workspace.root().join("atlas/annex.xlsx"))?;
    fs::copy(&outside, workspace.root().join("atlas/annex.xlsx"))?;
    fs::remove_file(workspace.root().join("nova/annex.xlsx"))?;
    symlink(
        workspace.root().join("atlas/annex.xlsx"),
        workspace.root().join("nova/annex.xlsx"),
    )?;
    assert!(workspace.analyze().is_err());
    Ok(())
}
