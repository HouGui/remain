use super::font::Font;
use crate::{config::Config, locale};
use anyhow::Result;
use lopdf::{
    Document, Object, Stream,
    content::{Content, Operation},
    dictionary,
};

pub(crate) const DEFAULT_FONT: &[u8] =
    include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

struct Line {
    text: String,
    size: f32,
    y: f32,
    entry: Option<usize>,
    warning: bool,
}

struct Layout {
    pages: Vec<Vec<Line>>,
    y: f32,
}

impl Layout {
    fn new() -> Self {
        Self {
            pages: vec![Vec::new()],
            y: 790.0,
        }
    }

    fn paragraph(
        &mut self,
        font: &Font<'_>,
        text: &str,
        size: f32,
        entry: Option<usize>,
        warning: bool,
    ) -> Result<()> {
        let width = if entry.is_some() { 440.0 } else { 499.0 };
        for (index, text) in font.wrap(text, size, width)?.into_iter().enumerate() {
            if self.y < 72.0 {
                self.pages.push(Vec::new());
                self.y = 790.0;
            }
            self.pages.last_mut().expect("contents page").push(Line {
                text,
                size,
                y: self.y,
                entry: entry.filter(|_| index == 0),
                warning,
            });
            self.y -= size + 5.0;
        }
        self.y -= 7.0;
        Ok(())
    }
}

/// Generate front matter, then graft untouched source pages after it. Page
/// numbers are PDF page positions, including every generated contents page.
pub(crate) fn assemble(
    config: &Config,
    scope: &str,
    entries: Vec<(String, Document)>,
    missing: &[String],
    kpi_coverage: (usize, usize),
    font_data: &[u8],
) -> Result<Document> {
    let font = Font::new(font_data)?;
    let t = |key| locale::text(&config.locale, &config.translations, key);
    let mut layout = Layout::new();
    layout.paragraph(&font, &config.title, 20.0, None, false)?;
    layout.paragraph(
        &font,
        &format!("{}: {} · {scope}", t("period"), config.period),
        11.0,
        None,
        false,
    )?;
    if !missing.is_empty() {
        layout.paragraph(&font, t("incomplete_report"), 15.0, None, true)?;
        layout.paragraph(&font, t("partial_explanation"), 10.0, None, false)?;
        layout.paragraph(&font, t("missing_submissions"), 12.0, None, true)?;
        for project in missing {
            layout.paragraph(&font, project, 10.0, None, false)?;
        }
    }
    if !config.metrics.is_empty() {
        layout.paragraph(
            &font,
            &format!(
                "{}: {} / {}",
                t("kpi_submissions"),
                kpi_coverage.0,
                kpi_coverage.1
            ),
            10.0,
            None,
            false,
        )?;
    }
    layout.paragraph(&font, t("contents"), 15.0, None, false)?;
    for (index, (label, _)) in entries.iter().enumerate() {
        layout.paragraph(&font, label, 11.0, Some(index), false)?;
    }
    let mut next = layout.pages.len() + 1;
    let numbers: Vec<_> = entries
        .iter()
        .map(|(_, doc)| {
            let number = next.to_string();
            next += doc.get_pages().len();
            number
        })
        .collect();
    let mut contents = Document::with_version("1.7");
    let root = contents.new_object_id();
    let embedded = font.embed(
        &mut contents,
        layout
            .pages
            .iter()
            .flatten()
            .flat_map(|line| line.text.chars())
            .chain("0123456789/".chars()),
    )?;
    let mut children = Vec::new();
    for (index, page) in layout.pages.iter().enumerate() {
        let mut operations = Vec::new();
        let mut text = |label: &str, x: f32, y: f32, size: f32, warning: bool| {
            operations.extend([
                Operation::new(
                    "rg",
                    if warning {
                        vec![0.65.into(), 0.2.into(), 0.05.into()]
                    } else {
                        vec![0.12.into(), 0.16.into(), 0.17.into()]
                    },
                ),
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), size.into()]),
                Operation::new("Td", vec![x.into(), y.into()]),
                Operation::new("Tj", vec![embedded.encode(label)]),
                Operation::new("ET", vec![]),
            ]);
        };
        for line in page {
            text(&line.text, 48.0, line.y, line.size, line.warning);
            if let Some(entry) = line.entry {
                let number = &numbers[entry];
                text(
                    number,
                    547.0 - font.width(number, line.size)?,
                    line.y,
                    line.size,
                    false,
                );
            }
        }
        let footer = format!("{}/{}", index + 1, layout.pages.len());
        text(
            &footer,
            547.0 - font.width(&footer, 10.0)?,
            40.0,
            10.0,
            false,
        );
        let stream = contents.add_object(Stream::new(
            dictionary! {},
            Content { operations }.encode()?,
        ));
        let page = contents
            .add_object(dictionary! {"Type" => "Page", "Parent" => root, "Contents" => stream});
        children.push(Object::Reference(page));
    }
    contents.objects.insert(root, dictionary! {"Type" => "Pages", "Count" => children.len() as i64, "Kids" => children, "MediaBox" => vec![0.into(),0.into(),595.into(),842.into()], "Resources" => dictionary! {"Font" => dictionary! {"F1" => embedded.id}}}.into());
    let catalog = contents.add_object(dictionary! {"Type" => "Catalog", "Pages" => root});
    contents.trailer.set("Root", catalog);
    super::merge(std::iter::once(contents).chain(entries.into_iter().map(|(_, document)| document)))
}
