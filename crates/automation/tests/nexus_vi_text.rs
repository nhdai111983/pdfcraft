//! NexusOffice (QC-PDF-01/02): Vietnamese written into a PDF — Add text, a watermark, a header
//! and footer — is drawn with an embedded subset of the bundled Unicode face, survives save and
//! reopen (text extraction through /ToUnicode gives it back), and is visibly rendered.

use std::path::{Path, PathBuf};

use pdfcraft_automation::{Automation, Content};
use serde_json::{Value, json};

const VI: &str = "Đặc biệt – Tiếng Việt có dấu: ẩ ặ ữ Ỹ";

/// A PDF with `n` blank 450×600 pt pages.
fn fixture(n: usize) -> Vec<u8> {
    let mut objs: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into()];
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + i)).collect();
    objs.push(format!("<< /Type /Pages /Kids [{}] /Count {n} /MediaBox [0 0 450 600] >>", kids.join(" ")));
    for _ in 0..n {
        objs.push("<< /Type /Page /Parent 2 0 R >>".into());
    }
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

/// A fresh directory with a two-page `blank.pdf`, removed when dropped.
struct Dir(PathBuf);
impl Dir {
    fn new(test: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("nexus-pdf-vi-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("blank.pdf"), fixture(2)).unwrap();
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn auto(dir: &Dir) -> Automation {
    Automation::new().with_root(dir.path()).unwrap()
}

fn ok(a: &mut Automation, tool: &str, args: Value) -> Value {
    match a.call(tool, &args) {
        Ok(mut c) => match c.remove(0) {
            Content::Json(v) => v,
            other => panic!("{tool}: expected JSON, got {other:?}"),
        },
        Err(e) => panic!("{tool} {args}: {e}"),
    }
}

fn page_text(a: &mut Automation, doc: u64) -> Vec<String> {
    ok(a, "text_extract", json!({ "doc": doc }))["pages"].as_array().unwrap().iter().map(|p| p["text"].as_str().unwrap().to_string()).collect()
}

/// Dark pixels in the band of rows `y0..y1` (pixels from the top, at 72 dpi).
fn ink(a: &mut Automation, doc: u64, page: usize, y0: u32, y1: u32) -> usize {
    let (w, h, px) = a.render_pixels(doc, page, 1.0).unwrap();
    assert_eq!((w, h), (450, 600));
    let px = px.as_ref();
    (y0..y1.min(h))
        .flat_map(|y| (0..w).map(move |x| ((y * w + x) * 4) as usize))
        .filter(|&i| px.get(i..i + 3).is_some_and(|p| p.iter().all(|v| *v < 160)))
        .count()
}

#[test]
fn vietnamese_text_watermark_and_header_round_trip_and_render() {
    let dir = Dir::new("roundtrip");
    let mut a = auto(&dir);
    let doc = ok(&mut a, "doc_open", json!({ "path": "blank.pdf" }))["doc"].as_u64().unwrap();
    // The default font is Helvetica: the text still goes in, drawn with the Unicode face.
    ok(&mut a, "page_add_text", json!({ "doc": doc, "page": 1, "text": VI, "at": [20, 120], "width": 410, "size": 14 }));
    // Decomposed input (as macOS and some keyboards type it) is drawn composed.
    ok(&mut a, "page_add_text", json!({ "doc": doc, "page": 1, "text": "Vie\u{0302}\u{0323}t Nam", "at": [20, 180], "width": 200, "bold": true }));
    ok(&mut a, "doc_watermark", json!({ "doc": doc, "text": VI, "font_size": 20, "rotation": 0, "opacity": 1, "color": "#000000" }));
    ok(
        &mut a,
        "doc_header_footer",
        json!({ "doc": doc, "header_center": VI, "footer_center": "Trang <<1>>/<<n>> — Công ty Ánh Dương", "font_size": 10 }),
    );
    ok(&mut a, "doc_save", json!({ "doc": doc, "path": "vi.pdf" }));

    let bytes = std::fs::read(dir.path().join("vi.pdf")).unwrap();
    // The file as written plus every stream inflated (dictionaries may sit in object streams).
    let mut readable = bytes.clone();
    let mut rest = bytes.as_slice();
    while let Some(start) = rest.windows(6).position(|w| w == b"stream") {
        let body = &rest[start + 6..];
        let body = body.strip_prefix(b"\r\n").or_else(|| body.strip_prefix(b"\n")).unwrap_or(body);
        let end = body.windows(9).position(|w| w == b"endstream").unwrap_or(body.len());
        let mut out = Vec::new();
        if std::io::Read::read_to_end(&mut flate2::read::ZlibDecoder::new(&body[..end]), &mut out).is_ok() {
            readable.extend_from_slice(&out);
        }
        // Past the `endstream`, so the next search doesn't take it for a `stream`.
        rest = body.get(end + 9..).unwrap_or_default();
    }
    let has = |needle: &[u8]| readable.windows(needle.len()).any(|w| w == needle);
    assert!(has(b"+Inter-Regular") && has(b"+Inter-SemiBold"), "subsets of Inter are embedded");
    assert!(has(b"/FontFile2") && has(b"/ToUnicode") && has(b"/Identity-H"));
    assert!(bytes.len() < 200_000, "subsets, not whole faces: {} bytes", bytes.len());

    // Reopened by another session, the text reads back exactly through /ToUnicode.
    let mut b = auto(&dir);
    let re = ok(&mut b, "doc_open", json!({ "path": "vi.pdf" }))["doc"].as_u64().unwrap();
    let texts = page_text(&mut b, re);
    assert!(texts[0].matches(VI).count() >= 3, "text, watermark and header on page 1: {:?}", texts[0]);
    assert!(texts[0].contains("Việt Nam"), "{:?}", texts[0]);
    assert!(texts[0].contains("Trang 1/2 — Công ty Ánh Dương"), "{:?}", texts[0]);
    assert!(texts[1].contains(VI) && texts[1].contains("Trang 2/2 — Công ty Ánh Dương"), "{:?}", texts[1]);
    let found = ok(&mut b, "text_find", json!({ "doc": re, "query": "Tiếng Việt" }));
    assert!(found["matches"].as_array().is_some_and(|m| m.len() >= 4), "{found}");
    // And it is drawn: ink in the header band, the text box, the watermark (page centre) and the
    // footer band, on a page that was blank.
    assert!(ink(&mut b, re, 1, 25, 45) > 50, "header");
    assert!(ink(&mut b, re, 1, 120, 145) > 100, "added text");
    assert!(ink(&mut b, re, 1, 285, 315) > 100, "watermark");
    assert!(ink(&mut b, re, 1, 555, 575) > 50, "footer");
    assert!(ink(&mut b, re, 2, 200, 280) == 0, "nothing where page 2 has nothing");
}

#[test]
fn a_chosen_inter_font_and_what_it_cannot_draw() {
    let dir = Dir::new("inter");
    let mut a = auto(&dir);
    let doc = ok(&mut a, "doc_open", json!({ "path": "blank.pdf" }))["doc"].as_u64().unwrap();
    // Plain ASCII in Inter, chosen outright, is Inter too (listed as such).
    ok(&mut a, "page_add_text", json!({ "doc": doc, "page": 1, "text": "Plain", "at": [20, 20], "font": "inter", "italic": true }));
    let list = ok(&mut a, "content_list", json!({ "doc": doc }));
    assert_eq!(list["items"][0]["font"], "Inter");
    // Characters no face here draws are still refused, with the character named.
    let err = a.call("doc_watermark", &json!({ "doc": doc, "text": "Đ 机密" })).unwrap_err().to_string();
    assert!(err.contains("U+673A"), "{err}");
    let err = a.call("page_add_text", &json!({ "doc": doc, "page": 1, "text": "Việt 中", "at": [20, 60] })).unwrap_err().to_string();
    assert!(err.contains("U+4E2D"), "{err}");
}
