//! NexusOffice: TrueType subsetting that keeps glyph ids ("retain GIDs"), for the bundled Unicode
//! text face written into PDFs (`unicode_face`). The glyphs a text needs, and the components of
//! its composite glyphs, keep their outlines; every other glyph becomes empty. Glyph ids don't
//! change, so `hmtx` and the hinting tables are copied as they are, the CIDs of an Identity
//! `CIDToGIDMap` are the glyph ids, and the subset is a valid font on its own. Layout tables
//! (`GSUB`, `GPOS`, `GDEF`) and glyph names (`post` becomes version 3) are dropped; `cmap` maps
//! only the subset's characters, so nothing takes an emptied glyph for a real one.
//!
//! Input is the bundled font, but every offset is still read with bounds checks: a font is data.

use std::collections::BTreeSet;

use skrifa::FontRef;
use skrifa::raw::types::Tag;

/// Tables copied unchanged when the source has them.
const COPIED: [&[u8; 4]; 10] = [b"OS/2", b"cvt ", b"fpgm", b"gasp", b"hhea", b"hmtx", b"maxp", b"name", b"prep", b"head"];

/// Most glyphs one subset keeps (with composite components): far above any page text's needs.
const MAX_GLYPHS: usize = 8192;

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    let b = data.get(at..at.checked_add(2)?)?;
    Some(u16::from_be_bytes([b[0], b[1]]))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    let b = data.get(at..at.checked_add(4)?)?;
    Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
}

/// The glyph ids a composite glyph's components refer to (ISO/IEC 14496-22 §5.3.3.2). A simple
/// or empty glyph has none.
fn components(glyph: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    // numberOfContours < 0 marks a composite glyph; its components start after the 10-byte header.
    if u16_at(glyph, 0).is_none_or(|n| (n as i16) >= 0) {
        return out;
    }
    let mut at = 10usize;
    // A glyph can't hold more components than it has bytes for; the bound keeps a hostile one finite.
    for _ in 0..glyph.len() / 4 {
        let (Some(flags), Some(id)) = (u16_at(glyph, at), u16_at(glyph, at.saturating_add(2))) else { break };
        out.push(id);
        at = at.saturating_add(4).saturating_add(if flags & 0x0001 != 0 { 4 } else { 2 });
        at = at.saturating_add(if flags & 0x0008 != 0 {
            2
        } else if flags & 0x0040 != 0 {
            4
        } else if flags & 0x0080 != 0 {
            8
        } else {
            0
        });
        if flags & 0x0020 == 0 {
            break;
        }
    }
    out
}

/// A `cmap` with one format 12 subtable (Windows, Unicode full repertoire) mapping `chars`.
fn cmap(chars: &[(char, u16)]) -> Vec<u8> {
    let mut groups: Vec<(u32, u16)> = chars.iter().map(|(c, g)| (u32::from(*c), *g)).collect();
    groups.sort_unstable();
    groups.dedup_by_key(|(c, _)| *c);
    let mut out = Vec::with_capacity(28 + groups.len() * 12);
    out.extend_from_slice(&0u16.to_be_bytes()); // version
    out.extend_from_slice(&1u16.to_be_bytes()); // one encoding record
    out.extend_from_slice(&3u16.to_be_bytes()); // Windows
    out.extend_from_slice(&10u16.to_be_bytes()); // Unicode full repertoire
    out.extend_from_slice(&12u32.to_be_bytes()); // subtable offset
    out.extend_from_slice(&12u16.to_be_bytes()); // format 12
    out.extend_from_slice(&0u16.to_be_bytes());
    let length = 16usize.saturating_add(groups.len().saturating_mul(12));
    out.extend_from_slice(&u32::try_from(length).unwrap_or(u32::MAX).to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes()); // language
    out.extend_from_slice(&u32::try_from(groups.len()).unwrap_or(u32::MAX).to_be_bytes());
    for (c, g) in groups {
        out.extend_from_slice(&c.to_be_bytes());
        out.extend_from_slice(&c.to_be_bytes());
        out.extend_from_slice(&u32::from(g).to_be_bytes());
    }
    out
}

fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0u32, |sum, w| {
        let mut b = [0u8; 4];
        for (d, s) in b.iter_mut().zip(w) {
            *d = *s;
        }
        sum.wrapping_add(u32::from_be_bytes(b))
    })
}

/// A subset of the TrueType font `bytes` that draws `chars` (each with its glyph id in that font).
/// Glyph 0 (`.notdef`) is always kept.
pub fn retain_gids(bytes: &[u8], chars: &[(char, u16)]) -> Result<Vec<u8>, String> {
    let font = FontRef::new(bytes).map_err(|_| "the text font is not a TrueType font".to_string())?;
    let table = |tag: &[u8; 4]| font.table_data(Tag::new(tag)).map(|d| d.as_bytes());
    let (Some(head), Some(maxp), Some(loca), Some(glyf)) = (table(b"head"), table(b"maxp"), table(b"loca"), table(b"glyf")) else {
        return Err("the text font has no TrueType outlines".into());
    };
    let long = u16_at(head, 50).ok_or("the text font's head table is too short")? == 1;
    let count = u16_at(maxp, 4).ok_or("the text font's maxp table is too short")?;
    let range = |g: u16| -> Option<(usize, usize)> {
        let g = usize::from(g);
        let (a, b) = if long {
            (u32_at(loca, g * 4)? as usize, u32_at(loca, g * 4 + 4)? as usize)
        } else {
            (usize::from(u16_at(loca, g * 2)?) * 2, usize::from(u16_at(loca, g * 2 + 2)?) * 2)
        };
        (a <= b && b <= glyf.len()).then_some((a, b))
    };
    // The glyphs to keep: the characters' and, transitively, their components.
    let mut keep = BTreeSet::new();
    let mut todo: Vec<u16> = std::iter::once(0).chain(chars.iter().map(|(_, g)| *g)).collect();
    while let Some(g) = todo.pop() {
        if g >= count || !keep.insert(g) {
            continue;
        }
        if keep.len() > MAX_GLYPHS {
            return Err("the text needs too many glyphs for one font".into());
        }
        if let Some((a, b)) = range(g) {
            todo.extend(glyf.get(a..b).map(components).unwrap_or_default());
        }
    }
    // glyf and a long loca: kept glyphs keep their data (padded to 4 bytes), the rest are empty.
    let mut new_glyf = Vec::new();
    let mut new_loca = Vec::with_capacity((usize::from(count) + 1) * 4);
    for g in 0..count {
        new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).map_err(|_| "the text font is too large")?.to_be_bytes());
        if keep.contains(&g)
            && let Some(data) = range(g).and_then(|(a, b)| glyf.get(a..b))
        {
            new_glyf.extend_from_slice(data);
            new_glyf.resize(new_glyf.len().next_multiple_of(4), 0);
        }
    }
    new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).map_err(|_| "the text font is too large")?.to_be_bytes());
    // post version 3: the 32-byte header without glyph names.
    let mut post = table(b"post").and_then(|p| p.get(..32)).map_or_else(|| vec![0; 32], <[u8]>::to_vec);
    if let Some(v) = post.get_mut(..4) {
        v.copy_from_slice(&0x0003_0000u32.to_be_bytes());
    }
    let mut tables: Vec<([u8; 4], Vec<u8>)> = COPIED.iter().filter_map(|tag| table(tag).map(|data| (**tag, data.to_vec()))).collect();
    for (tag, data) in &mut tables {
        if tag == b"head" {
            // Long loca offsets; the whole-font checksum adjustment is set last.
            if let Some(f) = data.get_mut(50..52) {
                f.copy_from_slice(&1u16.to_be_bytes());
            }
            if let Some(f) = data.get_mut(8..12) {
                f.copy_from_slice(&[0; 4]);
            }
        }
    }
    tables.push((*b"cmap", cmap(chars)));
    tables.push((*b"glyf", new_glyf));
    tables.push((*b"loca", new_loca));
    tables.push((*b"post", post));
    tables.sort_by_key(|a| a.0);
    // The table directory (OpenType §5.2), then each table on a 4-byte boundary.
    let n = u16::try_from(tables.len()).map_err(|_| "too many font tables")?;
    let power = 1u16 << (15 - n.leading_zeros().min(15));
    let search_range = power.saturating_mul(16);
    let entry_selector = power.trailing_zeros() as u16;
    let mut out = Vec::new();
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(&search_range.to_be_bytes());
    out.extend_from_slice(&entry_selector.to_be_bytes());
    out.extend_from_slice(&n.saturating_mul(16).saturating_sub(search_range).to_be_bytes());
    let mut offset = 12 + tables.len() * 16;
    let mut head_at = None;
    for (tag, data) in &tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(data).to_be_bytes());
        out.extend_from_slice(&u32::try_from(offset).map_err(|_| "the text font is too large")?.to_be_bytes());
        out.extend_from_slice(&u32::try_from(data.len()).map_err(|_| "the text font is too large")?.to_be_bytes());
        if tag == b"head" {
            head_at = Some(offset);
        }
        offset = offset.saturating_add(data.len().next_multiple_of(4));
    }
    for (_, data) in &tables {
        out.extend_from_slice(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    if let Some(at) = head_at {
        let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        if let Some(f) = out.get_mut(at + 8..at + 12) {
            f.copy_from_slice(&adjustment.to_be_bytes());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::MetadataProvider;
    use skrifa::outline::{DrawSettings, OutlinePen};
    use skrifa::prelude::{LocationRef, Size};

    #[derive(Default)]
    struct Count(usize);
    impl OutlinePen for Count {
        fn move_to(&mut self, _: f32, _: f32) {
            self.0 += 1;
        }
        fn line_to(&mut self, _: f32, _: f32) {
            self.0 += 1;
        }
        fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
            self.0 += 1;
        }
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
            self.0 += 1;
        }
        fn close(&mut self) {}
    }

    fn points(font: &FontRef, g: u16) -> usize {
        let mut pen = Count::default();
        let glyph = font.outline_glyphs().get(skrifa::GlyphId::new(u32::from(g))).unwrap();
        glyph.draw(DrawSettings::unhinted(Size::unscaled(), LocationRef::default()), &mut pen).unwrap();
        pen.0
    }

    #[test]
    fn the_subset_keeps_the_texts_glyphs_with_their_components_and_nothing_else() {
        let full = FontRef::new(crate::INTER_REGULAR).unwrap();
        let text = "Đặc biệt Ỹ";
        let chars: Vec<(char, u16)> = text.chars().map(|c| (c, full.charmap().map(c).unwrap().to_u32() as u16)).collect();
        let bytes = retain_gids(crate::INTER_REGULAR, &chars).unwrap();
        assert!(bytes.len() < crate::INTER_REGULAR.len() / 4, "a subset, not the whole face: {} bytes", bytes.len());
        crate::embedding::validate(&bytes).unwrap();
        let sub = FontRef::new(&bytes).unwrap();
        assert!(sub.table_data(Tag::new(b"GPOS")).is_none());
        for (c, g) in &chars {
            assert_eq!(sub.charmap().map(*c).map(|g| g.to_u32() as u16), Some(*g), "{c} keeps its glyph id");
            assert_eq!(points(&sub, *g), points(&full, *g), "{c} draws as in the full face");
            assert!(*c == ' ' || points(&sub, *g) > 0, "{c} has ink");
        }
        // A character the text doesn't use is neither mapped nor drawn.
        let x = full.charmap().map('X').unwrap().to_u32() as u16;
        assert_eq!(sub.charmap().map('X'), None);
        assert_eq!(points(&sub, x), 0);
        // The whole-font checksum is right (OpenType: the sum of the file is 0xB1B0AFBA).
        assert_eq!(checksum(&bytes), 0xB1B0_AFBA);
    }

    #[test]
    fn hostile_input_is_refused_without_panicking() {
        assert!(retain_gids(b"", &[]).is_err());
        assert!(retain_gids(b"not a font at all", &[('a', 1)]).is_err());
        // Glyph ids beyond the font are ignored, not indexed.
        assert!(retain_gids(crate::INTER_REGULAR, &[('a', u16::MAX)]).is_ok());
        // A composite glyph whose components run off its end stops at the end.
        assert_eq!(components(&[0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x20, 0x00]), Vec::<u16>::new());
        assert_eq!(components(&[0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0x20, 0x00, 0x05, 0, 0]), vec![5]);
    }
}
