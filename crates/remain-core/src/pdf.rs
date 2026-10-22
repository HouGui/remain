use anyhow::{Context, Result, ensure};
mod contents;
mod font;
pub(crate) use contents::{DEFAULT_FONT, assemble};
use lopdf::{Document, Object, dictionary};
use std::{collections::HashSet, path::Path};

pub(crate) fn read(path: &Path) -> Result<Document> {
    let doc =
        Document::load(path).with_context(|| format!("Cannot read PDF {}", path.display()))?;
    validate(&doc).with_context(|| format!("PDF {}", path.display()))?;
    Ok(doc)
}

fn validate(doc: &Document) -> Result<()> {
    ensure!(
        !doc.is_encrypted(),
        "Encrypted PDFs are unsupported; export an unencrypted copy"
    );
    let catalog = doc.catalog()?;
    ensure!(
        !catalog.has(b"AcroForm"),
        "Interactive forms/signatures are unsupported; export a static, unsigned PDF copy"
    );
    ensure!(!doc.get_pages().is_empty(), "PDF has no readable pages");
    for id in doc.get_pages().values() {
        let page = doc.get_dictionary(*id).context("Invalid page dictionary")?;
        ensure!(
            inherited(doc, *id, b"MediaBox")?.is_some(),
            "Page has no inherited MediaBox"
        );
        if let Ok(contents) = page.get(b"Contents") {
            let mut bytes = Vec::new();
            content(doc, contents, &mut bytes, 0)?;
            lopdf::content::Content::decode(&bytes).context("Cannot decode page content")?;
        }
        for annotation in doc.get_page_annotations(*id)? {
            ensure!(
                annotation.get(b"Subtype").and_then(Object::as_name).ok() != Some(b"Widget"),
                "Interactive page widgets are unsupported; export a static PDF copy"
            );
        }
    }
    Ok(())
}

/// Graft each original page tree under a fresh root. Retaining the full source
/// tree preserves inherited media boxes, rotations, and resources. A fresh
/// catalog deliberately omits source-level bookmarks and document actions.
pub(crate) fn merge(documents: impl IntoIterator<Item = Document>) -> Result<Document> {
    let mut output = Document::with_version("1.7");
    let root = output.new_object_id();
    let mut children = Vec::new();
    let mut count: i64 = 0;
    for mut doc in documents {
        count += doc.get_pages().len() as i64;
        // Explicit effective page attributes also support readers that do not
        // resolve inline resource dictionaries on an ancestor page-tree node.
        for id in doc.get_pages().values() {
            for key in [b"Resources".as_slice(), b"MediaBox", b"CropBox", b"Rotate"] {
                if let Some(value) = inherited(&doc, *id, key)? {
                    doc.get_dictionary_mut(*id)?.set(key, value);
                }
            }
        }
        doc.renumber_objects_with(output.max_id + 1);
        let pages = doc.catalog()?.get(b"Pages")?.as_reference()?;
        doc.get_object_mut(pages)?
            .as_dict_mut()?
            .set("Parent", root);
        children.push(Object::Reference(pages));
        output.max_id = output.max_id.max(doc.max_id);
        output.objects.extend(doc.objects);
    }
    ensure!(count > 0, "No PDF pages to merge");
    output.objects.insert(
        root,
        dictionary! { "Type" => "Pages", "Kids" => children, "Count" => count }.into(),
    );
    let catalog = output.add_object(dictionary! { "Type" => "Catalog", "Pages" => root });
    output.trailer.set("Root", catalog);
    output.prune_objects();
    output.compress();
    Ok(output)
}

fn inherited(doc: &Document, mut id: lopdf::ObjectId, key: &[u8]) -> Result<Option<Object>> {
    let mut visited = HashSet::new();
    loop {
        ensure!(
            visited.insert(id) && visited.len() <= 256,
            "Invalid or cyclic PDF page tree"
        );
        let node = doc.get_dictionary(id)?;
        if let Ok(value) = node.get(key) {
            return Ok(Some(value.clone()));
        }
        match node.get(b"Parent") {
            Ok(parent) => id = parent.as_reference()?,
            Err(_) => return Ok(None),
        }
    }
}

fn content(doc: &Document, object: &Object, output: &mut Vec<u8>, depth: usize) -> Result<()> {
    ensure!(depth < 32, "Invalid PDF content reference chain");
    match object {
        Object::Null => {}
        Object::Reference(id) => content(doc, doc.get_object(*id)?, output, depth + 1)?,
        Object::Array(items) => {
            for item in items {
                content(doc, item, output, depth + 1)?;
            }
        }
        Object::Stream(stream) => {
            output.extend(
                stream
                    .get_plain_content()
                    .context("Invalid PDF content stream")?,
            );
            output.push(b'\n');
        }
        _ => anyhow::bail!("Invalid PDF page Contents"),
    }
    Ok(())
}
