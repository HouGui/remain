use anyhow::{Context, Result, ensure};
use lopdf::{Document, Object, ObjectId, Stream, dictionary};
use std::collections::BTreeMap;

pub(super) struct Font<'a> {
    data: &'a [u8],
    face: ttf_parser::Face<'a>,
}

impl<'a> Font<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let face = ttf_parser::Face::parse(data, 0).context("Cannot read contents font")?;
        ensure!(
            face.tables().glyf.is_some(),
            "Contents font must have TrueType outlines"
        );
        Ok(Self { data, face })
    }

    fn glyph(&self, character: char) -> Result<ttf_parser::GlyphId> {
        self.face.glyph_index(character).with_context(|| format!(
            "Contents font has no glyph for {character:?} (U+{:04X}); set contents_font to a TrueType font with this character", character as u32))
    }

    pub fn width(&self, text: &str, size: f32) -> Result<f32> {
        text.chars()
            .map(|c| {
                self.glyph(c)
                    .map(|g| self.face.glyph_hor_advance(g).unwrap_or(0) as f32)
            })
            .sum::<Result<f32>>()
            .map(|width| width * size / self.face.units_per_em() as f32)
    }

    pub fn wrap(&self, text: &str, size: f32, width: f32) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        let mut line = String::new();
        let mut used = 0.0;
        let space = self.width(" ", size)?;
        for word in text.split_whitespace() {
            let word_width = self.width(word, size)?;
            let separator = if line.is_empty() { 0.0 } else { space };
            if used + separator + word_width <= width {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
                used += separator + word_width;
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            used = 0.0;
            for character in word.chars() {
                let glyph_width = self.width(character.encode_utf8(&mut [0; 4]), size)?;
                if !line.is_empty() && used + glyph_width > width {
                    lines.push(std::mem::take(&mut line));
                    used = 0.0;
                }
                line.push(character);
                used += glyph_width;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        Ok(lines)
    }

    pub fn embed(
        &self,
        document: &mut Document,
        text: impl Iterator<Item = char>,
    ) -> Result<Embedded> {
        let mut characters = BTreeMap::new();
        for character in text {
            characters.entry(character).or_insert(0u16);
        }
        ensure!(
            characters.len() < u16::MAX as usize,
            "Too many distinct characters on contents pages"
        );
        let mut mapping = vec![0u8, 0];
        let mut widths = Vec::new();
        let scale = 1000.0 / self.face.units_per_em() as f32;
        let mut unicode = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /REMAINUnicode def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        let entries: Vec<_> = characters.keys().copied().collect();
        for (chunk_index, chunk) in entries.chunks(100).enumerate() {
            unicode.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for (offset, character) in chunk.iter().enumerate() {
                let cid = (chunk_index * 100 + offset + 1) as u16;
                let glyph = self.glyph(*character)?;
                mapping.extend_from_slice(&glyph.0.to_be_bytes());
                widths.push(Object::Integer(
                    (self.face.glyph_hor_advance(glyph).unwrap_or(0) as f32 * scale).round() as i64,
                ));
                let encoded = character
                    .encode_utf16(&mut [0; 2])
                    .iter()
                    .map(|u| format!("{u:04X}"))
                    .collect::<String>();
                unicode.push_str(&format!("<{cid:04X}> <{encoded}>\n"));
            }
            unicode.push_str("endbfchar\n");
        }
        for (index, cid) in characters.values_mut().enumerate() {
            *cid = (index + 1) as u16;
        }
        unicode.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        let font_file = document.add_object(Stream::new(
            dictionary! {"Length1" => self.data.len() as i64},
            self.data.to_vec(),
        ));
        let bounds = self.face.global_bounding_box();
        let descriptor = document.add_object(dictionary! {
            "Type" => "FontDescriptor", "FontName" => "REMAINContents", "Flags" => 32,
            "FontBBox" => vec![bounds.x_min,bounds.y_min,bounds.x_max,bounds.y_max].into_iter().map(|v| Object::Real(v as f32 * scale)).collect::<Vec<_>>(),
            "ItalicAngle" => 0, "Ascent" => self.face.ascender() as f32 * scale,
            "Descent" => self.face.descender() as f32 * scale, "CapHeight" => self.face.capital_height().unwrap_or(self.face.ascender()) as f32 * scale,
            "StemV" => 80, "FontFile2" => font_file
        });
        let gid_map = document.add_object(Stream::new(dictionary! {}, mapping));
        let cid_font = document.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => "REMAINContents",
            "CIDSystemInfo" => dictionary! {"Registry" => Object::string_literal("Adobe"), "Ordering" => Object::string_literal("Identity"), "Supplement" => 0},
            "FontDescriptor" => descriptor, "CIDToGIDMap" => gid_map,
            "W" => vec![Object::Integer(1),Object::Array(widths)]
        });
        let to_unicode = document.add_object(Stream::new(dictionary! {}, unicode.into_bytes()));
        let id = document.add_object(dictionary! {"Type" => "Font", "Subtype" => "Type0", "BaseFont" => "REMAINContents", "Encoding" => "Identity-H", "DescendantFonts" => vec![Object::Reference(cid_font)], "ToUnicode" => to_unicode});
        Ok(Embedded { id, characters })
    }
}

pub(super) struct Embedded {
    pub id: ObjectId,
    characters: BTreeMap<char, u16>,
}

impl Embedded {
    pub fn encode(&self, text: &str) -> Object {
        Object::String(
            text.chars()
                .flat_map(|c| self.characters[&c].to_be_bytes())
                .collect(),
            lopdf::StringFormat::Hexadecimal,
        )
    }
}
