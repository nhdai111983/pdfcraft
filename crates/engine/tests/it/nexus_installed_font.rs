//! NexusOffice (Nexus PDF Pro, P1 "giữ đúng phông"): a font installed on the user's machine can
//! be loaded into a page so edited text keeps its face, whatever licence the font declares, as
//! long as the font's own embedding permissions allow it.
use crate::support;
use pdfcraft_cos::{Dict, Document, Object, SaveOptions, Stream};
use pdfcraft_engine::{BlockStyle, Edit, Session};
use std::sync::Arc;

fn document() -> Arc<Vec<u8>> {
    let mut d = Document::new_empty();
    let pages = d.add(Object::Dict(Dict::new()));
    let contents = d.add(Object::Stream(Stream::flate(Dict::new(), b"BT /F1 18 Tf 30 160 Td (Source) Tj ET")));
    let mut font = Dict::new();
    font.set(b"Type".to_vec(), Object::name("Font"));
    font.set(b"Subtype".to_vec(), Object::name("Type1"));
    font.set(b"BaseFont".to_vec(), Object::name("Helvetica"));
    let mut fonts = Dict::new();
    fonts.set(b"F1".to_vec(), Object::Dict(font));
    let mut res = Dict::new();
    res.set(b"Font".to_vec(), Object::Dict(fonts));
    let mut p = Dict::new();
    p.set(b"Type".to_vec(), Object::name("Page"));
    p.set(b"Parent".to_vec(), Object::Ref(pages));
    p.set(b"Contents".to_vec(), Object::Ref(contents));
    p.set(b"Resources".to_vec(), Object::Dict(res));
    p.set(b"MediaBox".to_vec(), Object::Array([0, 0, 300, 240].map(Object::Int).to_vec()));
    let page = d.add(Object::Dict(p));
    d.update_dict(pages, |p| {
        p.set(b"Type".to_vec(), Object::name("Pages"));
        p.set(b"Kids".to_vec(), Object::Array(vec![Object::Ref(page)]));
        p.set(b"Count".to_vec(), Object::Int(1));
    })
    .unwrap();
    let mut root = Dict::new();
    root.set(b"Type".to_vec(), Object::name("Catalog"));
    root.set(b"Pages".to_vec(), Object::Ref(pages));
    let root = d.add(Object::Dict(root));
    d.trailer_mut().set(b"Root".to_vec(), Object::Ref(root));
    Arc::new(pdfcraft_cos::write_full(&d, &SaveOptions::default()).unwrap())
}

/// The rectangle font with a proprietary licence description, as an installed system font has
/// ("MIT License" and "Proprietary" are both 11 characters, so the name table keeps its layout).
fn proprietary_font() -> Vec<u8> {
    let font = support::font_fixture::cff_font();
    let utf16 = |s: &str| s.encode_utf16().flat_map(u16::to_be_bytes).collect::<Vec<u8>>();
    let (from, to) = (utf16("MIT License"), utf16("Proprietary"));
    let at = font.windows(from.len()).position(|w| w == from.as_slice()).expect("the fixture declares its licence");
    let mut out = font;
    out[at..at + to.len()].copy_from_slice(&to);
    out
}

/// `font` with one more table: the directory gains a record (sorted by tag) and every existing
/// table moves down 16 bytes.
fn with_table(font: &[u8], tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let be16 = |at: usize| u16::from_be_bytes([font[at], font[at + 1]]);
    let be32 = |at: usize| u32::from_be_bytes([font[at], font[at + 1], font[at + 2], font[at + 3]]);
    let num = usize::from(be16(4));
    let mut records: Vec<([u8; 4], u32, u32, u32)> = (0..num)
        .map(|i| {
            let r = 12 + 16 * i;
            ([font[r], font[r + 1], font[r + 2], font[r + 3]], be32(r + 4), be32(r + 8) + 16, be32(r + 12))
        })
        .collect();
    let mut out = font[..12].to_vec();
    out[4..6].copy_from_slice(&((num + 1) as u16).to_be_bytes());
    let body = &font[12 + 16 * num..];
    let mut end = 12 + 16 * (num + 1) + body.len();
    end += (4 - end % 4) % 4;
    records.push((*tag, 0, end as u32, data.len() as u32));
    records.sort_by_key(|r| r.0);
    for (tag, sum, offset, len) in &records {
        out.extend_from_slice(tag);
        out.extend_from_slice(&sum.to_be_bytes());
        out.extend_from_slice(&offset.to_be_bytes());
        out.extend_from_slice(&len.to_be_bytes());
    }
    out.extend_from_slice(body);
    out.resize(end, 0);
    out.extend_from_slice(data);
    out
}

/// An `OS/2` table, version 0 (78 bytes), with this `fsType`.
fn os2(fs_type: u16) -> Vec<u8> {
    let mut t = vec![0u8; 78];
    t[8..10].copy_from_slice(&fs_type.to_be_bytes());
    t
}

#[test]
fn an_installed_font_is_loaded_whatever_licence_it_declares() {
    let font = proprietary_font();
    let mut s = Session::new();
    let id = s.open("source.pdf", None, document(), None).unwrap();
    // The project's own path asks for an open licence, as upstream's asset policy says.
    assert!(s.apply(id, Edit::RegisterTextFont { page: 0, bytes: Arc::new(font.clone()) }).is_err());
    // The user's installed font is theirs: loaded, and usable as the paragraph's face.
    s.apply(id, Edit::RegisterInstalledFont { page: 0, bytes: Arc::new(font.clone()) }).unwrap();
    let resource = s.get(id).unwrap().text_editor_fonts(0).into_iter().find(|f| f.bytes == font).unwrap().resource;
    let style = BlockStyle { resource: Some(resource), width: Some(240.0), ..Default::default() };
    s.apply(id, Edit::EditTextBlock { page: 0, block: 0, text: "A A".into(), style }).unwrap();
    let lines = s.get(id).unwrap().text_lines(0);
    assert_eq!(lines.iter().map(|l| l.text.as_str()).collect::<String>(), "A A");
    assert!(lines.iter().all(|l| l.base_font == "RectFont-Regular"), "{:?}", lines.iter().map(|l| &l.base_font).collect::<Vec<_>>());
    // Saved and reopened, the text is still there in that face.
    let saved = s.save_bytes(id).unwrap();
    let re = s.open("saved.pdf", None, saved, None).unwrap();
    assert_eq!(s.get(re).unwrap().text_blocks(0)[0].editor_text, "A A");
    // Undo takes the font back out with the edit before it.
    s.undo(id).unwrap();
    s.undo(id).unwrap();
    assert!(s.get(id).unwrap().text_editor_fonts(0).into_iter().all(|f| f.bytes != font));
}

#[test]
fn an_installed_font_that_forbids_embedding_is_refused() {
    let base = proprietary_font();
    let mut s = Session::new();
    let id = s.open("source.pdf", None, document(), None).unwrap();
    // Restricted (0x0002), preview & print (0x0004) and bitmap-only (0x0200) are refused;
    // installable (0) and editable (0x0008) are loaded.
    for (fs_type, allowed) in [(0x0002, false), (0x0004, false), (0x0200, false), (0x0000, true), (0x0008, true)] {
        let font = with_table(&base, b"OS/2", &os2(fs_type));
        let r = s.apply(id, Edit::RegisterInstalledFont { page: 0, bytes: Arc::new(font) });
        assert_eq!(r.is_ok(), allowed, "fsType {fs_type:#06x}: {r:?}");
    }
    // Not a font at all.
    assert!(s.apply(id, Edit::RegisterInstalledFont { page: 0, bytes: Arc::new(b"not a font".to_vec()) }).is_err());
}
