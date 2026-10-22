use super::*;

#[test]
fn classified_configuration_renders_all_sections_and_roundtrips() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create_classified(tmp.path().join("classes"), "fr")?;
    let workspace = Workspace::load(&path)?;
    let mut editor = Editor::new(&workspace, path.clone(), "fr");
    let context = egui::Context::default();
    for section in [
        Section::Workspace,
        Section::Metrics,
        Section::Documents,
        Section::Groups,
        Section::Categories,
    ] {
        editor.section = section;
        let output = context.run(Default::default(), |context| {
            editor.show(context);
        });
        assert!(!output.shapes.is_empty());
    }
    let class = &mut editor.config.metrics[0]
        .breakdown
        .as_mut()
        .expect("breakdown")
        .classes[0];
    class.label = "Articles de conférence".into();
    editor.save()?;
    let saved = Workspace::load(path)?;
    assert_eq!(
        saved.config().indicators()[1].label(),
        "Publications · Articles de conférence"
    );
    assert_eq!(saved.analyze()?.overall[1], Some(6.));
    Ok(())
}

#[test]
fn invalid_form_cannot_replace_valid_configuration() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let workspace = Workspace::load(&path)?;
    let before = std::fs::read(&path)?;
    let mut editor = Editor::new(&workspace, path.clone(), "en");
    editor.config.groups.clear();
    assert!(editor.save().is_err());
    assert_eq!(std::fs::read(path)?, before);
    Ok(())
}

#[test]
fn save_preserves_names_and_detects_external_edits() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "de")?;
    let workspace = Workspace::load(&path)?;
    let mut editor = Editor::new(&workspace, path.clone(), "de");
    editor.config.period = "2027".into();
    editor.save()?;
    let loaded = Workspace::load(&path)?;
    assert_eq!(loaded.config().period, "2027");
    assert_eq!(loaded.config().title, workspace.config().title);
    // The old form is now stale: a second save must not overwrite this edit.
    assert!(editor.save().is_err());
    Ok(())
}

#[test]
fn french_editor_saves_labels_without_renaming_source_headers() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "fr")?;
    let workspace = Workspace::load(&path)?;
    let mut editor = Editor::new(&workspace, path.clone(), "fr");
    editor.config.locale = "fr-CA".into();
    editor.config.title = "Bilan de l’année".into();
    let context = egui::Context::default();
    let output = context.run(Default::default(), |context| {
        editor.show(context);
    });
    assert!(!output.shapes.is_empty());
    editor.save()?;
    let saved = Workspace::load(path)?;
    assert_eq!(saved.config().title, "Bilan de l’année");
    assert_eq!(
        saved.config().annex.as_ref().expect("KPI settings").sheet,
        "Indicateurs"
    );
    assert_eq!(saved.analyze()?.overall[0], Some(12.));
    Ok(())
}

#[test]
fn changes_before_opening_the_editor_cannot_be_overwritten() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let workspace = Workspace::load(&path)?;
    let mut changed = workspace.config().clone();
    changed.period = "external edit".into();
    let bytes = serde_json::to_vec_pretty(&changed)?;
    std::fs::write(&path, &bytes)?;
    let editor = Editor::new(&workspace, path.clone(), "en");
    assert!(editor.save().is_err());
    assert_eq!(std::fs::read(path)?, bytes);
    Ok(())
}

#[test]
fn omitted_defaults_do_not_look_like_external_changes() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let mut json: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    json["metrics"][0]
        .as_object_mut()
        .expect("metric")
        .remove("unit");
    json["metrics"][0]
        .as_object_mut()
        .expect("metric")
        .remove("decimals");
    std::fs::write(&path, serde_json::to_vec(&json)?)?;
    let workspace = Workspace::load(&path)?;
    let editor = Editor::new(&workspace, path, "en");
    editor.save()?;
    Ok(())
}

#[test]
fn interface_and_report_language_changes_preserve_names_and_source_mappings() -> anyhow::Result<()>
{
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create_classified(tmp.path().join("demo"), "en")?;
    let workspace = Workspace::load(&path)?;
    let mut editor = Editor::new(&workspace, path, "en");
    let mut expected = serde_json::to_value(&editor.config)?;
    let context = egui::Context::default();
    for language in ["en", "de", "fr"] {
        editor.language = language.into();
        for locale in ["en", "de", "fr-CA"] {
            editor.config.locale = locale.into();
            expected["locale"] = locale.into();
            for section in [
                Section::Workspace,
                Section::Metrics,
                Section::Documents,
                Section::Groups,
                Section::Categories,
            ] {
                editor.section = section;
                let _ = context.run(Default::default(), |context| {
                    editor.show(context);
                });
            }
            assert_eq!(serde_json::to_value(&editor.config)?, expected);
        }
    }
    editor.save()?;
    assert_eq!(
        Workspace::load(&editor.path)?.analyze()?.overall[0],
        Some(12.)
    );
    assert_eq!(
        fields::next_id("group", ["GROUP-1", "group-3"].into_iter()),
        "group-2"
    );
    Ok(())
}

#[test]
fn new_workspace_starts_with_an_empty_importable_roster_and_one_pdf_rule() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let mut editor = Editor::empty(tmp.path().into(), "en");
    editor.config.period = "2026".into();
    editor.save()?;
    let workspace = Workspace::load(&editor.path)?;
    assert!(workspace.projects().is_empty());
    assert!(workspace.config().metrics.is_empty());
    assert_eq!(workspace.config().inputs.documents.len(), 1);
    assert!(workspace.config().inputs.documents[0].required);
    assert!(editor.save().is_err());
    Ok(())
}
