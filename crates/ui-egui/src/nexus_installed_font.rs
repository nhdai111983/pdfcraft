//! NexusOffice (Nexus PDF Pro, P1 "giữ đúng phông"): keep a paragraph's own face when an edit
//! types letters its embedded subset doesn't have.
//!
//! A PDF usually embeds only the letters the document used (a *subset*, `ABCDEF+Face`). Typing a
//! letter it lacks (a Vietnamese `ặ`, say) makes the rewrite fall back to another font, so the
//! edited paragraph changes typeface. When the host window knows a full copy of the same face
//! installed on this machine ([`crate::nexus_wps::Host::installed_font`]), the copy is loaded
//! into the page first (`Edit::RegisterInstalledFont`, which checks the font's own embedding
//! permissions) and the paragraph is written in it: same face, every letter.
//!
//! Which font the host offers, and whether it offers one at all, is its business (in
//! NexusOffice, a Pro feature). Without a host hook nothing changes.

use std::collections::BTreeSet;
use std::sync::Arc;

use pdfcraft_engine::{DocId, Edit};

use crate::PdfCraftApp;

/// The face a font is: its `BaseFont` without a subset's `ABCDEF+` tag.
pub(crate) fn face_of(base_font: &str) -> &str {
    match base_font.split_once('+') {
        Some((tag, face)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => face,
        _ => base_font,
    }
}

/// The letters of `draft` that `original` doesn't have (spaces and line breaks aside): what a
/// subset made for `original` may not draw.
pub(crate) fn new_letters(original: &str, draft: &str) -> Vec<char> {
    let had: BTreeSet<char> = original.chars().collect();
    let mut out: Vec<char> = draft.chars().filter(|c| !c.is_whitespace() && !had.contains(c)).collect();
    out.sort_unstable();
    out.dedup();
    out
}

impl PdfCraftApp {
    /// `edit`, written in a full installed copy of its paragraph's face when it types letters the
    /// paragraph didn't have and the host offers that face. Any other edit, or one the user gave a
    /// font of their own, comes back unchanged; so does one when loading the font fails (the
    /// rewrite then falls back as before).
    pub(crate) fn nexus_keep_face(&mut self, id: DocId, edit: Edit) -> Edit {
        let Edit::EditTextBlock { page, block, text, mut style } = edit else { return edit };
        let unchanged = |text, style| Edit::EditTextBlock { page, block, text, style };
        let user_font = style.resource.is_some() || style.family.is_some() || style.ranges.iter().any(|r| r.resource.is_some() || r.family.is_some());
        let Some(find) = self.host.installed_font.as_ref().filter(|_| !user_font) else { return unchanged(text, style) };
        let Some(doc) = self.session.get(id) else { return unchanged(text, style) };
        let Some(source) = doc.text_blocks(page).get(block).cloned() else { return unchanged(text, style) };
        let lines = doc.text_lines(page);
        let faces: BTreeSet<&str> = source.lines.iter().filter_map(|i| lines.get(*i)).map(|l| face_of(&l.base_font)).collect();
        // One face only: a paragraph mixing faces keeps the rewrite's own choice.
        let face = match faces.iter().collect::<Vec<_>>().as_slice() {
            [one] => (**one).to_owned(),
            _ => return unchanged(text, style),
        };
        let missing = new_letters(&source.editor_text, &text);
        if missing.is_empty() {
            return unchanged(text, style);
        }
        // A copy loaded by an earlier edit on this page is used again.
        let loaded = |doc: &pdfcraft_engine::Document, bytes: &[u8]| {
            doc.text_editor_fonts(page).into_iter().find(|f| f.bytes.as_slice() == bytes).map(|f| f.resource)
        };
        let Some(bytes) = find(&face, &missing) else { return unchanged(text, style) };
        let resource = match loaded(doc, &bytes) {
            Some(r) => Some(r),
            None => match self.session.apply(id, Edit::RegisterInstalledFont { page, bytes: Arc::clone(&bytes) }) {
                Ok(()) => self.session.get(id).and_then(|d| loaded(d, &bytes)),
                Err(e) => {
                    log::warn!("installed font {face} not loaded: {e}");
                    None
                }
            },
        };
        style.resource = resource;
        unchanged(text, style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_subset_tag_is_not_the_face() {
        assert_eq!(face_of("ABCDEF+TimesNewRomanPSMT"), "TimesNewRomanPSMT");
        assert_eq!(face_of("TimesNewRomanPSMT"), "TimesNewRomanPSMT");
        assert_eq!(face_of("Abcdef+Face"), "Abcdef+Face", "a tag is six capitals");
        assert_eq!(face_of("ABC+Face"), "ABC+Face");
    }

    /// One page: "Source" in Helvetica (not embedded), as resource F1.
    fn helvetica_pdf() -> Vec<u8> {
        let content = "BT /F1 18 Tf 30 160 Td (Source) Tj ET";
        let objs = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 240] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
            format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
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

    /// Henry 03/11 (Pro P1): typing Vietnamese into a paragraph whose font lacks it writes the
    /// paragraph in the installed copy the host offers for that face; the host is asked for the
    /// face and the new letters only, once; without the hook nothing is asked.
    #[test]
    fn an_edit_typing_new_letters_uses_the_installed_copy_of_its_face() {
        let inter: Arc<Vec<u8>> = Arc::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf").to_vec());
        type Asked = std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<char>)>>>;
        let asked: Asked = Default::default();
        let mut app = crate::PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("a.pdf", None, helvetica_pdf()).unwrap();
        let (seen, font) = (std::rc::Rc::clone(&asked), Arc::clone(&inter));
        app.host.installed_font = Some(Box::new(move |face: &str, letters: &[char]| {
            seen.borrow_mut().push((face.to_owned(), letters.to_vec()));
            Some(Arc::clone(&font))
        }));
        let id = app.views[0].id;
        let edit = |text: &str| Edit::EditTextBlock { page: 0, block: 0, text: text.into(), style: Default::default() };
        // Nothing new typed: the host isn't asked.
        assert!(app.apply_edit(edit("Source")));
        assert!(asked.borrow().is_empty());
        assert!(app.apply_edit(edit("Sửa nguồn")));
        assert_eq!(asked.borrow().as_slice(), &[("Helvetica".to_owned(), vec!['a', 'g', 'n', 'ồ', 'ử'])]);
        let lines = app.session.get(id).unwrap().text_lines(0);
        assert_eq!(lines.iter().map(|l| l.text.as_str()).collect::<String>(), "Sửa nguồn");
        assert!(lines.iter().all(|l| l.base_font == "Inter-Regular"), "{:?}", lines.iter().map(|l| &l.base_font).collect::<Vec<_>>());
        // A font the user picked wins: the host isn't asked.
        asked.borrow_mut().clear();
        let style = pdfcraft_engine::BlockStyle { family: Some((pdfcraft_engine::FontFamily::Times, false, false)), ..Default::default() };
        let _ = app.apply_edit(Edit::EditTextBlock { page: 0, block: 0, text: "Sửa nguồn mới".into(), style });
        assert!(asked.borrow().is_empty());
    }

    /// Manual check with a real document and a real installed font:
    /// `NEXUS_PROBE_PDF=<pdf> NEXUS_PROBE_FONT=<ttf> NEXUS_PROBE_TEXT=<new text> cargo test -p
    /// pdfcraft-ui-egui --lib real_document -- --ignored --nocapture` edits the first paragraph of
    /// page 1, once without the hook and once with it, and prints the faces used.
    #[test]
    #[ignore = "needs a real PDF and an installed font"]
    fn real_document() {
        let (Some(pdf), Some(font)) = (std::env::var_os("NEXUS_PROBE_PDF"), std::env::var_os("NEXUS_PROBE_FONT")) else { return };
        let text = std::env::var("NEXUS_PROBE_TEXT").unwrap_or_else(|_| "Hợp đồng số 12".into());
        let bytes = std::fs::read(pdf).unwrap();
        let font: Arc<Vec<u8>> = Arc::new(std::fs::read(font).unwrap());
        for hooked in [false, true] {
            let mut app = crate::PdfCraftApp::new();
            app.set_option("language", "en").unwrap();
            app.open_bytes("probe.pdf", None, bytes.clone()).unwrap();
            if hooked {
                let f = Arc::clone(&font);
                app.host.installed_font = Some(Box::new(move |face: &str, letters: &[char]| {
                    println!("asked for {face} {letters:?}");
                    Some(Arc::clone(&f))
                }));
            }
            let id = app.views[0].id;
            let before: Vec<String> = app.session.get(id).unwrap().text_lines(0).iter().map(|l| l.base_font.clone()).collect();
            let ok = app.apply_edit(Edit::EditTextBlock { page: 0, block: 0, text: text.clone(), style: Default::default() });
            let doc = app.session.get(id).unwrap();
            let after: Vec<(String, String)> = doc.text_lines(0).iter().map(|l| (l.text.clone(), l.base_font.clone())).collect();
            println!("hook={hooked} ok={ok} before={before:?}\n  after={after:?}");
            if hooked {
                let out = std::env::var("NEXUS_PROBE_OUT").unwrap_or_else(|_| "probe-out.pdf".into());
                std::fs::write(out, app.session.save_bytes(id).unwrap().as_slice()).unwrap();
            }
        }
    }

    #[test]
    fn only_letters_the_paragraph_lacked_count() {
        assert_eq!(new_letters("Hop dong so 12", "Hợp đồng số 12"), vec!['đ', 'ố', 'ồ', 'ợ']);
        assert!(new_letters("Hợp đồng", "Hợp  đồng\n").is_empty());
        assert!(new_letters("abc", "").is_empty());
    }
}
