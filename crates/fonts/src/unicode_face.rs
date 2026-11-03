//! NexusOffice (QC-PDF-01/02): the bundled Unicode text face for page text the standard 14 fonts
//! can't encode. Inter (OFL-1.1, already bundled as the interface font and drawn into typed
//! signatures) covers Latin Extended with all of Vietnamese, Greek and Cyrillic. Text written with
//! it embeds a subset of the face ([`crate::subset`]) as a Type 0 font (CIDFontType2, Identity-H,
//! CID = glyph id) with a `/ToUnicode` map, so it shows, copies and searches in every viewer
//! without installed fonts. Nothing from a system font is ever embedded (AGENTS.md §1.4).
//!
//! Text is drawn composed (NFC), so typed decomposed Vietnamese (`e` + U+0302 + U+0301) shows the
//! precomposed glyph instead of loose marks; what the caller stores stays as typed.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use pdfcraft_cos::{Dict, Document, ObjRef, Object, PdfString, Stream};
use skrifa::instance::{LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};

/// Inter SemiBold: bold text in the Unicode face.
pub static INTER_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf");

/// The family name shown for the face.
pub const FAMILY: &str = "Inter";

/// Most distinct characters one embedded font maps (a page header, watermark or text box needs
/// a few dozen).
const MAX_CHARS: usize = 4096;

/// The face's program: Regular, or SemiBold for bold text.
pub fn face(bold: bool) -> &'static [u8] {
    if bold { INTER_SEMIBOLD } else { crate::INTER_REGULAR }
}

/// `s` composed (Unicode NFC), as it is drawn.
pub fn compose(s: &str) -> Cow<'_, str> {
    icu_normalizer::ComposingNormalizerBorrowed::new_nfc().normalize(s)
}

fn font(bold: bool) -> Option<FontRef<'static>> {
    FontRef::new(face(bold)).ok()
}

/// Whether the face draws `c` (tabs are drawn as spaces). Line breaks are not drawable.
pub fn has(c: char) -> bool {
    let c = if c == '\t' { ' ' } else { c };
    // Both weights cover the same characters; Regular answers for both.
    font(false).is_some_and(|f| f.charmap().map(c).is_some_and(|g| g.to_u32() != 0))
}

/// The first character of `s` (composed) the face can't draw, skipping `\n`.
pub fn first_missing(s: &str) -> Option<char> {
    compose(s).chars().find(|c| *c != '\n' && !has(*c))
}

/// Advance of `s` (composed) at `size` points. Characters the face lacks measure nothing.
pub fn width(s: &str, size: f64, bold: bool) -> f64 {
    let Some(f) = font(bold) else { return 0.0 };
    let upem = f64::from(f.metrics(Size::unscaled(), LocationRef::default()).units_per_em.max(1));
    let (map, metrics) = (f.charmap(), f.glyph_metrics(Size::unscaled(), LocationRef::default()));
    let units: f64 =
        compose(s).chars().filter_map(|c| map.map(if c == '\t' { ' ' } else { c })).map(|g| f64::from(metrics.advance_width(g).unwrap_or(0.0))).sum();
    units / upem * size
}

/// A subset of the face written into a document, and how to show text with it.
#[derive(Clone, Debug, PartialEq)]
pub struct UnicodeFont {
    /// The Type 0 font dictionary.
    pub font: ObjRef,
    /// Character → CID (= glyph id).
    codes: BTreeMap<char, u16>,
    /// The CID of the space, for [`UnicodeFont::show_spaced`].
    space: Option<u16>,
    /// Advance of the space in text space units (1000 per em).
    space_width: f64,
}

impl UnicodeFont {
    fn hex(&self, s: &str) -> String {
        compose(s).chars().filter_map(|c| self.codes.get(&if c == '\t' { ' ' } else { c })).fold(String::new(), |mut out, cid| {
            let _ = write!(out, "{cid:04X}");
            out
        })
    }

    /// `s` as a string operand for `Tj` (`<…>`). Characters the font doesn't map are left out:
    /// callers check [`first_missing`] first.
    pub fn show(&self, s: &str) -> Vec<u8> {
        format!("<{}>", self.hex(s)).into_bytes()
    }

    /// `s` as a `TJ` array whose spaces are each widened by `extra` points at `size` points (the
    /// `Tw` operator only stretches the single-byte code 32, never a two-byte CID).
    pub fn show_spaced(&self, s: &str, extra: f64, size: f64) -> Vec<u8> {
        if !(extra.is_finite() && extra > 0.0 && size.is_finite() && size > 0.0) || self.space.is_none() {
            let mut out = self.show(s);
            out.extend_from_slice(b" Tj");
            return out;
        }
        // TJ shifts are thousandths of an em, subtracted from the advance.
        let shift = -(extra / size * 1000.0).round() as i64;
        let mut out = String::from("[");
        for (i, word) in compose(s).split([' ', '\t']).enumerate() {
            if i > 0 {
                let _ = write!(out, "<{:04X}> {} ", self.space.unwrap_or(0), shift);
            }
            let _ = write!(out, "<{}> ", self.hex(word));
        }
        out.push_str("] TJ");
        out.into_bytes()
    }

    /// The width of a space in points at `size` (for justified text).
    pub fn space_width(&self, size: f64) -> f64 {
        self.space_width / 1000.0 * size
    }
}

/// Write a subset of the face that draws every character of `texts` (composed) into `doc`.
/// Fails, writing nothing, when a character isn't in the face or the text is far too varied.
pub fn embed<'a>(doc: &mut Document, texts: impl IntoIterator<Item = &'a str>, bold: bool) -> Result<UnicodeFont, String> {
    let bytes = face(bold);
    let f = FontRef::new(bytes).map_err(|_| "the Unicode text font is unreadable".to_string())?;
    let map = f.charmap();
    let mut chars = BTreeMap::new();
    for text in texts {
        for c in compose(text).chars().filter(|c| !matches!(c, '\n' | '\r')) {
            let c = if c == '\t' { ' ' } else { c };
            let g = map.map(c).map(|g| g.to_u32()).filter(|g| *g != 0).ok_or_else(|| format!("the {FAMILY} font can't draw \"{c}\""))?;
            chars.insert(c, u16::try_from(g).map_err(|_| "the Unicode text font has too many glyphs".to_string())?);
            if chars.len() > MAX_CHARS {
                return Err("the text uses too many different characters for one font".into());
            }
        }
    }
    let list: Vec<(char, u16)> = chars.iter().map(|(c, g)| (*c, *g)).collect();
    let program = crate::subset::retain_gids(bytes, &list)?;
    let m = f.metrics(Size::unscaled(), LocationRef::default());
    let scale = 1000.0 / f64::from(m.units_per_em.max(1));
    let advances = f.glyph_metrics(Size::unscaled(), LocationRef::default());
    let advance = |g: u16| f64::from(advances.advance_width(skrifa::GlyphId::new(u32::from(g))).unwrap_or(0.0)) * scale;

    // A subset tag (ISO 32000-2 §9.6.4) derived from the glyphs, so equal subsets are named alike.
    let hash = list.iter().fold(0x811c_9dc5u32, |h, (_, g)| (h ^ u32::from(*g)).wrapping_mul(0x0100_0193));
    let tag: String = (0..6).map(|i| char::from(b'A' + ((hash >> (i * 5)) % 26) as u8)).collect();
    let postscript = if bold { "Inter-SemiBold" } else { "Inter-Regular" };
    let name = format!("{tag}+{postscript}");

    // ToUnicode: one entry per glyph; when two characters share one, the first is what copies.
    let mut unicode: BTreeMap<u16, char> = BTreeMap::new();
    for (c, g) in &list {
        unicode.entry(*g).or_insert(*c);
    }
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &char)> = unicode.iter().collect();
    for block in entries.chunks(100) {
        let _ = writeln!(cmap, "{} beginbfchar", block.len());
        for (g, c) in block {
            let hex: String = c.encode_utf16(&mut [0; 2]).iter().map(|u| format!("{u:04X}")).collect();
            let _ = writeln!(cmap, "<{g:04X}> <{hex}>");
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");

    let mut file = Dict::new();
    file.set(b"Length1".to_vec(), Object::Int(i64::try_from(program.len()).unwrap_or(i64::MAX)));
    let file = doc.add(Object::Stream(Stream::flate(file, &program)));
    let mut descriptor = Dict::new();
    descriptor.set(b"Type".to_vec(), Object::name("FontDescriptor"));
    descriptor.set(b"FontName".to_vec(), Object::name(&name));
    descriptor.set(b"FontFamily".to_vec(), Object::String(PdfString::literal(FAMILY.as_bytes().to_vec())));
    descriptor.set(b"Flags".to_vec(), Object::Int(4));
    descriptor.set(b"ItalicAngle".to_vec(), Object::Int(0));
    descriptor.set(b"Ascent".to_vec(), Object::Real((f64::from(m.ascent) * scale).round()));
    descriptor.set(b"Descent".to_vec(), Object::Real((f64::from(m.descent) * scale).round()));
    descriptor.set(b"CapHeight".to_vec(), Object::Real((f64::from(m.cap_height.unwrap_or(m.ascent)) * scale).round()));
    descriptor.set(b"StemV".to_vec(), Object::Int(if bold { 120 } else { 80 }));
    let bbox = m.bounds.map_or([0.0, f64::from(m.descent), 1000.0, f64::from(m.ascent)], |b| {
        [f64::from(b.x_min), f64::from(b.y_min), f64::from(b.x_max), f64::from(b.y_max)]
    });
    descriptor.set(b"FontBBox".to_vec(), Object::Array(bbox.iter().map(|v| Object::Real((v * scale).round())).collect()));
    descriptor.set(b"FontFile2".to_vec(), Object::Ref(file));
    let descriptor = doc.add(Object::Dict(descriptor));

    let mut system = Dict::new();
    system.set(b"Registry".to_vec(), Object::String(PdfString::literal(b"Adobe".to_vec())));
    system.set(b"Ordering".to_vec(), Object::String(PdfString::literal(b"Identity".to_vec())));
    system.set(b"Supplement".to_vec(), Object::Int(0));
    let mut widths = Vec::new();
    for g in unicode.keys() {
        widths.push(Object::Int(i64::from(*g)));
        widths.push(Object::Array(vec![Object::Real((advance(*g) * 1000.0).round() / 1000.0)]));
    }
    let mut cid = Dict::new();
    cid.set(b"Type".to_vec(), Object::name("Font"));
    cid.set(b"Subtype".to_vec(), Object::name("CIDFontType2"));
    cid.set(b"BaseFont".to_vec(), Object::name(&name));
    cid.set(b"CIDSystemInfo".to_vec(), Object::Dict(system));
    cid.set(b"FontDescriptor".to_vec(), Object::Ref(descriptor));
    cid.set(b"CIDToGIDMap".to_vec(), Object::name("Identity"));
    cid.set(b"DW".to_vec(), Object::Int(0));
    cid.set(b"W".to_vec(), Object::Array(widths));
    let cid = doc.add(Object::Dict(cid));
    let to_unicode = doc.add(Object::Stream(Stream::flate(Dict::new(), cmap.as_bytes())));
    let mut top = Dict::new();
    top.set(b"Type".to_vec(), Object::name("Font"));
    top.set(b"Subtype".to_vec(), Object::name("Type0"));
    top.set(b"BaseFont".to_vec(), Object::name(&name));
    top.set(b"Encoding".to_vec(), Object::name("Identity-H"));
    top.set(b"DescendantFonts".to_vec(), Object::Array(vec![Object::Ref(cid)]));
    top.set(b"ToUnicode".to_vec(), Object::Ref(to_unicode));
    let font = doc.add(Object::Dict(top));
    let space = chars.get(&' ').copied();
    Ok(UnicodeFont { font, codes: chars, space, space_width: space.map_or(0.0, advance) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::Metrics;

    const VI: &str = "Đặc biệt – Tiếng Việt có dấu: ẩ ặ ữ Ỹ";

    #[test]
    fn the_face_draws_all_of_vietnamese_in_both_weights() {
        let all = "ẮẰẲẴẶ ắằẳẵặ ẤẦẨẪẬ ấầẩẫậ Đđ ỆệỞởỮữỰựỸỹ ƠơƯư — Nguyễn Thị Ngọc Ánh, Hà Nội, Đắk Lắk";
        assert_eq!(first_missing(all), None);
        assert_eq!(first_missing(VI), None);
        // Decomposed input composes before it's looked up.
        let nfd = "Vie\u{0302}\u{0323}t";
        assert_eq!(first_missing(nfd), None);
        assert_eq!(width(nfd, 12.0, false), width("Việt", 12.0, false));
        for c in all.chars().filter(|c| *c != ' ') {
            assert!(FontRef::new(INTER_SEMIBOLD).unwrap().charmap().map(c).is_some(), "SemiBold draws {c}");
        }
        assert_eq!(first_missing("中"), Some('中'));
        assert_eq!(first_missing("a\rb"), Some('\r'));
        assert!(width("Đ", 10.0, false) > 0.0);
    }

    #[test]
    fn embedded_text_round_trips_through_to_unicode() {
        let mut doc = Document::new_empty();
        let f = embed(&mut doc, [VI, "Vie\u{0302}\u{0323}t"], false).unwrap();
        let dict = doc.dict(&Object::Ref(f.font)).unwrap();
        let m = Metrics::from_dict(&doc, &dict);
        let shown = f.show(VI);
        let codes: Vec<u8> =
            (0..(shown.len() - 2) / 2).map(|i| u8::from_str_radix(std::str::from_utf8(&shown[1 + i * 2..3 + i * 2]).unwrap(), 16).unwrap()).collect();
        assert_eq!(m.decode(&codes), VI);
        assert!(m.base_font.ends_with("+Inter-Regular"), "{}", m.base_font);
        // The embedded program is a valid subset that still draws the text.
        let program = crate::pdf::embedded_sfnt(&doc, &dict).unwrap();
        assert!(program.len() < crate::INTER_REGULAR.len() / 4);
        assert!(crate::embedding::covers(&program, VI));
        // Justified text widens spaces with TJ shifts.
        let tj = String::from_utf8(f.show_spaced("a b", 2.0, 10.0)).unwrap();
        assert!(tj.starts_with('[') && tj.ends_with("] TJ") && tj.contains("-200"), "{tj}");
        // What the face lacks is refused before anything is written.
        let before = doc.object_numbers().len();
        assert!(embed(&mut doc, ["Đ 中"], false).is_err());
        assert_eq!(doc.object_numbers().len(), before);
    }
}
