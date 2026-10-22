//! PDF structure and unsupported-input boundary tests.
mod common;
use anyhow::Result;
use lopdf::{Document, Object, dictionary};
use std::fs;

#[test]
fn inherited_resources_media_boxes_and_rotation_survive_merging() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let source = workspace.root().join("atlas/part-1.pdf");
    let mut original = Document::load(&source)?;
    let pages = original.catalog()?.get(b"Pages")?.as_reference()?;
    original.get_dictionary_mut(pages)?.set("Rotate", 90);
    original.save(&source)?;
    let output = tmp.path().join("report");
    workspace.export(&output)?;
    let doc = Document::load(output.join("report.pdf"))?;
    let page = *doc.get_pages().get(&2).expect("first page");
    let parent = doc.get_dictionary(page)?.get(b"Parent")?.as_reference()?;
    let tree = doc.get_dictionary(parent)?;
    assert_eq!(tree.get(b"Rotate")?.as_i64()?, 90);
    assert!(tree.has(b"MediaBox") && tree.has(b"Resources"));
    assert!(doc.extract_text(&[2])?.contains("ATLAS"));
    Ok(())
}

#[test]
fn interactive_forms_are_rejected_instead_of_losing_field_data() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let path = workspace.root().join("atlas/part-1.pdf");
    let mut doc = Document::load(&path)?;
    let catalog = doc.trailer.get(b"Root")?.as_reference()?;
    doc.get_dictionary_mut(catalog)?
        .set("AcroForm", dictionary! { "Fields" => Vec::<Object>::new() });
    doc.save(path)?;
    let error = workspace.analyze().expect_err("form unsupported");
    assert!(format!("{error:#}").contains("Interactive forms"));
    Ok(())
}

#[test]
fn corrupt_and_zero_page_pdfs_fail_with_file_context() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let path = workspace.root().join("atlas/part-1.pdf");
    fs::write(&path, b"%PDF-1.7\nbroken")?;
    assert!(format!("{:#}", workspace.analyze().expect_err("corrupt")).contains("atlas"));
    let mut doc = Document::with_version("1.7");
    let pages = doc.add_object(
        dictionary! { "Type" => "Pages", "Kids" => Vec::<Object>::new(), "Count" => 0 },
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
    doc.trailer.set("Root", catalog);
    doc.save(path)?;
    assert!(format!("{:#}", workspace.analyze().expect_err("empty")).contains("no readable pages"));
    Ok(())
}

#[test]
fn blank_pages_are_valid_but_invalid_contents_are_not() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let path = workspace.root().join("atlas/part-1.pdf");
    let mut doc = Document::load(&path)?;
    let page = *doc.get_pages().get(&1).expect("first page");
    doc.get_dictionary_mut(page)?.set("Contents", Object::Null);
    doc.save(&path)?;
    assert_eq!(workspace.analyze()?.projects[0].pages, 1);
    doc.get_dictionary_mut(page)?
        .set("Contents", Object::Integer(42));
    doc.save(&path)?;
    assert!(
        format!("{:#}", workspace.analyze().expect_err("invalid content"))
            .contains("Invalid PDF page Contents")
    );
    Ok(())
}
