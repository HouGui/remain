use anyhow::Result;
use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};
use std::path::Path;

pub(super) fn create(path: &Path, project: &str, part: usize, locale: &str) -> Result<()> {
    let mut doc = Document::with_version("1.7");
    let pages = doc.new_object_id();
    let font = doc.add_object(
        dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding" },
    );
    let bold = doc.add_object(
        dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica-Bold", "Encoding" => "WinAnsiEncoding" },
    );
    let (part_label, paragraphs) = match locale {
        "de" => (
            "Teil",
            [
                "Fiktiver Projektbericht. Berichtszeitraum: 2026.",
                "Ergebnisse und Ausblick",
                "Das Team hat neue Lernmodule und Materialtests entwickelt.",
                "Die Excel-Anlage enthält synthetische Kennzahlen.",
                "Alle Namen, Texte und Zahlen sind frei erfunden.",
            ],
        ),
        "fr" => (
            "Partie",
            [
                "Rapport de projet fictif. Période : 2026.",
                "Résultats et prochaines étapes",
                "L'équipe a créé des modules et testé des matériaux.",
                "L'annexe Excel contient des indicateurs fictifs.",
                "Tous les noms, textes et chiffres sont inventés.",
            ],
        ),
        _ => (
            "Part",
            [
                "Fictional project report. Reporting period: 2026.",
                "Results and next steps",
                "The team developed learning modules and material trials.",
                "The Excel annex contains synthetic performance indicators.",
                "All names, narrative text, and numbers are invented.",
            ],
        ),
    };
    let lines = [
        ("F2", 14, 790, "REMAIN / DEMO".into()),
        (
            "F2",
            26,
            720,
            format!("{} / {part_label} {part}", project.to_uppercase()),
        ),
        ("F1", 12, 675, paragraphs[0].into()),
        ("F2", 16, 610, paragraphs[1].into()),
        ("F1", 12, 575, paragraphs[2].into()),
        ("F1", 12, 550, paragraphs[3].into()),
        ("F1", 12, 525, paragraphs[4].into()),
        ("F1", 10, 65, "Synthetic fixture / MIT / REMAIN".into()),
    ];
    let mut operations = Vec::new();
    for (font, size, y, text) in lines {
        // These fixture strings use the Latin-1 subset of WinAnsi. Encode each
        // character once so accents survive without bundling a font file.
        let bytes = text
            .chars()
            .map(u8::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        operations.extend([
            Operation::new("BT", vec![]),
            Operation::new(
                "Tf",
                vec![Object::Name(font.as_bytes().to_vec()), size.into()],
            ),
            Operation::new("Td", vec![50.into(), y.into()]),
            Operation::new("Tj", vec![Object::string_literal(bytes)]),
            Operation::new("ET", vec![]),
        ]);
    }
    let content = doc.add_object(Stream::new(
        dictionary! {},
        Content { operations }.encode()?,
    ));
    let page =
        doc.add_object(dictionary! { "Type" => "Page", "Parent" => pages, "Contents" => content });
    // Inherited resources/media box exercise the merge implementation.
    doc.objects.insert(pages, dictionary! { "Type" => "Pages", "Kids" => vec![Object::Reference(page)], "Count" => 1,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()], "Resources" => dictionary! { "Font" => dictionary! { "F1" => font, "F2" => bold } } }.into());
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages });
    doc.trailer.set("Root", catalog);
    doc.compress();
    doc.save(path)?;
    Ok(())
}
