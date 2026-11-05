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

/// The undo label of loading an installed font; the step is joined with the edit it serves.
const LOAD_INSTALLED_FONT: &str = "Load installed font";

/// Whether `bytes` is a copy of `face` (a `BaseFont` without its subset tag) and not some other
/// font: one of its own names is the face, compared as a subset's name is compared with a full
/// face ([`pdfcraft_fonts::embedding::face_key`]: letters and digits, case folded). Its PostScript
/// name (`TimesNewRomanPS-BoldMT`), its full name (`Arial Bold` for `Arial,Bold`), or its family
/// with its style (only the family for a regular style). A host offering another family, or the
/// regular face for a bold one, is not taken: the paragraph would change typeface, which is what
/// this hook exists to prevent.
pub(crate) fn is_same_face(face: &str, bytes: &[u8]) -> bool {
    use pdfcraft_fonts::embedding::face_key;
    use skrifa::MetadataProvider as _;
    use skrifa::string::StringId;
    // `validate`'s own limit: nothing larger is ever loaded.
    if bytes.len() > 16 << 20 {
        return false;
    }
    let Ok(font) = skrifa::FontRef::new(bytes) else { return false };
    let want = face_key(face);
    if want.is_empty() {
        return false;
    }
    let names = |id| -> Vec<String> { font.localized_strings(id).take(64).map(|s| face_key(&s.to_string())).collect() };
    let regular = |style: &str| style.is_empty() || ["regular", "normal", "roman", "book", "plain"].contains(&style);
    let mut keys = names(StringId::POSTSCRIPT_NAME);
    keys.extend(names(StringId::FULL_NAME));
    for (family, style) in
        [(StringId::FAMILY_NAME, StringId::SUBFAMILY_NAME), (StringId::TYPOGRAPHIC_FAMILY_NAME, StringId::TYPOGRAPHIC_SUBFAMILY_NAME)]
    {
        for f in names(family) {
            for s in names(style) {
                if regular(&s) {
                    keys.push(f.clone());
                }
                keys.push(format!("{f}{s}"));
            }
        }
    }
    keys.contains(&want)
}

impl PdfCraftApp {
    /// `edit`, written in a full installed copy of its paragraph's face when it types letters the
    /// paragraph didn't have and the host offers that face; and whether the copy was loaded into
    /// the page for it now (`Load installed font`, an undo step the caller joins with the edit's,
    /// or takes back when the edit fails: [`Self::nexus_after_keep_face`]). Any other edit, or one
    /// the user gave a font of their own, comes back unchanged; so does one when the host offers
    /// no copy, a font that isn't that face or lacks the letters, or loading it fails (the rewrite
    /// then falls back as before).
    pub(crate) fn nexus_keep_face(&mut self, id: DocId, edit: Edit) -> (Edit, bool) {
        let Edit::EditTextBlock { page, block, text, mut style } = edit else { return (edit, false) };
        let unchanged = |text, style| (Edit::EditTextBlock { page, block, text, style }, false);
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
        // The host's answer is input: the same face, with the letters, or nothing.
        let letters: String = missing.iter().collect();
        if !is_same_face(&face, &bytes) || !pdfcraft_fonts::embedding::covers(&bytes, &letters) {
            log::warn!("the installed font offered for {face} is another face or lacks the new letters; not used");
            return unchanged(text, style);
        }
        if let Some(r) = loaded(doc, &bytes) {
            style.resource = Some(r);
            return unchanged(text, style);
        }
        if let Err(e) = self.session.apply(id, Edit::RegisterInstalledFont { page, bytes: Arc::clone(&bytes) }) {
            log::warn!("installed font {face} not loaded: {e}");
            return unchanged(text, style);
        }
        match self.session.get(id).and_then(|d| loaded(d, &bytes)) {
            Some(r) => {
                style.resource = Some(r);
                (Edit::EditTextBlock { page, block, text, style }, true)
            }
            None => {
                // Loaded but not offered for text: nothing would use it.
                self.session.nexus_discard_last_step(id, LOAD_INSTALLED_FONT);
                unchanged(text, style)
            }
        }
    }

    /// After the edit [`Self::nexus_keep_face`] loaded a font for (`loaded`): one undo step for
    /// both when it applied, and no font left behind (nor a Redo for it) when it failed.
    pub(crate) fn nexus_after_keep_face(&mut self, id: DocId, loaded: bool, applied: bool) {
        if !loaded {
            return;
        }
        if applied {
            self.session.nexus_join_last_steps(id, LOAD_INSTALLED_FONT);
        } else {
            self.session.nexus_discard_last_step(id, LOAD_INSTALLED_FONT);
        }
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

    /// One page: "Source" in `base_font` (not embedded), as resource F1.
    fn source_pdf(base_font: &str) -> Vec<u8> {
        let content = "BT /F1 18 Tf 30 160 Td (Source) Tj ET";
        let objs = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 240] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
            format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
            format!("<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} >>"),
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

    fn inter() -> Arc<Vec<u8>> {
        Arc::new(include_bytes!("../../../assets/fonts/Inter-Regular.ttf").to_vec())
    }

    /// The host's installed copy of Inter: the same face, but not byte for byte the program the
    /// page or the app already has (so loading it, and using it, can be told apart from the
    /// rewrite's own fallback, which is Inter too).
    fn installed_inter() -> Arc<Vec<u8>> {
        let mut bytes = inter().to_vec();
        bytes.extend_from_slice(&[0; 4]);
        Arc::new(bytes)
    }

    /// One page: "Source" in a full embedded Inter (`BaseFont /Inter-Regular`).
    fn inter_pdf() -> Vec<u8> {
        let mut s = pdfcraft_engine::Session::new();
        let id = s.open("s.pdf", None, Arc::new(source_pdf("Helvetica")), None).unwrap();
        s.apply(id, Edit::RegisterTextFont { page: 0, bytes: inter() }).unwrap();
        let resource = s.get(id).unwrap().text_editor_fonts(0).into_iter().find(|f| f.bytes == *inter()).unwrap().resource;
        let style = pdfcraft_engine::BlockStyle { resource: Some(resource), ..Default::default() };
        s.apply(id, Edit::EditTextBlock { page: 0, block: 0, text: "Source".into(), style }).unwrap();
        s.save_bytes(id).unwrap().to_vec()
    }

    type Asked = std::rc::Rc<std::cell::RefCell<Vec<(String, Vec<char>)>>>;

    /// An app showing `pdf` whose host offers `font` for any face, and what the host was asked.
    fn hooked_app(pdf: Vec<u8>, font: Arc<Vec<u8>>) -> (crate::PdfCraftApp, DocId, Asked) {
        let asked: Asked = Default::default();
        let mut app = crate::PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("a.pdf", None, pdf).unwrap();
        let seen = std::rc::Rc::clone(&asked);
        app.host.installed_font = Some(Box::new(move |face: &str, letters: &[char]| {
            seen.borrow_mut().push((face.to_owned(), letters.to_vec()));
            Some(Arc::clone(&font))
        }));
        let id = app.views[0].id;
        (app, id, asked)
    }

    fn edit(text: &str) -> Edit {
        Edit::EditTextBlock { page: 0, block: 0, text: text.into(), style: Default::default() }
    }

    fn text(app: &crate::PdfCraftApp, id: DocId) -> String {
        app.session.get(id).unwrap().text_lines(0).iter().map(|l| l.text.as_str()).collect()
    }

    /// The page resource holding exactly `font`, if the page has it.
    fn resource_of(app: &crate::PdfCraftApp, id: DocId, font: &[u8]) -> Option<String> {
        app.session.get(id).unwrap().text_editor_fonts(0).into_iter().find(|f| f.bytes == font).map(|f| f.resource)
    }

    /// Whether every line of the page is written in exactly `font`.
    fn written_in(app: &crate::PdfCraftApp, id: DocId, font: &[u8]) -> bool {
        let lines = app.session.get(id).unwrap().text_lines(0);
        resource_of(app, id, font).is_some_and(|r| !lines.is_empty() && lines.iter().all(|l| l.font == r))
    }

    /// Henry 03/11 (Pro P1): typing Vietnamese into a paragraph whose font lacks it writes the
    /// paragraph in the installed copy the host offers for that face; the host is asked for the
    /// face and the new letters only, once; without new letters nothing is asked.
    #[test]
    fn an_edit_typing_new_letters_uses_the_installed_copy_of_its_face() {
        let font = installed_inter();
        let (mut app, id, asked) = hooked_app(inter_pdf(), Arc::clone(&font));
        // Nothing new typed: the host isn't asked.
        assert!(app.apply_edit(edit("Source")));
        assert!(asked.borrow().is_empty());
        assert!(app.apply_edit(edit("Sửa nguồn")));
        assert_eq!(asked.borrow().as_slice(), &[("Inter-Regular".to_owned(), vec!['a', 'g', 'n', 'ồ', 'ử'])]);
        assert_eq!(text(&app, id), "Sửa nguồn");
        assert!(written_in(&app, id, &font), "the paragraph isn't written in the installed copy");
        // A font the user picked wins: the host isn't asked.
        asked.borrow_mut().clear();
        let style = pdfcraft_engine::BlockStyle { family: Some((pdfcraft_engine::FontFamily::Times, false, false)), ..Default::default() };
        let _ = app.apply_edit(Edit::EditTextBlock { page: 0, block: 0, text: "Sửa nguồn mới".into(), style });
        assert!(asked.borrow().is_empty());
    }

    /// Review 05/11: the host's font is input. Offered another family for the paragraph's face
    /// (Inter for Helvetica), the edit doesn't take it: the paragraph would change typeface, which
    /// is what the hook is for preventing. Nothing is loaded into the page either.
    #[test]
    fn an_installed_font_of_another_face_is_not_used() {
        let font = installed_inter();
        let (mut app, id, asked) = hooked_app(source_pdf("Helvetica"), Arc::clone(&font));
        assert!(app.apply_edit(edit("Sửa nguồn")));
        assert_eq!(asked.borrow().first().map(|a| a.0.as_str()), Some("Helvetica"), "the host was asked");
        assert_eq!(text(&app, id), "Sửa nguồn");
        assert_eq!(resource_of(&app, id, &font), None, "another face was loaded into the page");
        // Not a font at all: the edit goes through as without the hook.
        let (mut app, id, _) = hooked_app(inter_pdf(), Arc::new(b"not a font".to_vec()));
        assert!(app.apply_edit(edit("Sửa nguồn")));
        assert_eq!(text(&app, id), "Sửa nguồn");
    }

    /// Review 05/11: loading the installed copy and the edit it serves are one action: one Undo
    /// takes both back (the font too), one Redo brings both.
    #[test]
    fn one_undo_takes_back_the_edit_and_the_font_it_loaded() {
        let font = installed_inter();
        let (mut app, id, _) = hooked_app(inter_pdf(), Arc::clone(&font));
        assert!(app.apply_edit(edit("Sửa nguồn")));
        assert!(written_in(&app, id, &font));
        let label = app.session.get(id).unwrap().undo_label().cloned().unwrap();
        assert_eq!(label.wording, "Edit text");
        app.session.undo(id).unwrap();
        assert_eq!(text(&app, id), "Source");
        assert_eq!(resource_of(&app, id, &font), None, "the font stayed after the edit was undone");
        assert!(app.session.undo(id).is_err(), "a second step was left to undo");
        app.session.redo(id).unwrap();
        assert_eq!(text(&app, id), "Sửa nguồn");
        assert!(written_in(&app, id, &font));
    }

    /// Review 05/11: the copy is the face itself, by one of its own names, never another family or
    /// another style of it.
    #[test]
    fn the_same_face_by_its_own_names() {
        let font = inter();
        for face in ["Inter-Regular", "Inter", "InterRegular", "Inter Regular", "inter-regular"] {
            assert!(is_same_face(face, &font), "{face}");
        }
        for face in ["Inter,Bold", "Inter-Bold", "InterItalic", "Helvetica", "ArialMT", "", "+"] {
            assert!(!is_same_face(face, &font), "{face}");
        }
        assert!(!is_same_face("Inter-Regular", b"not a font"));
        assert!(!is_same_face("Inter-Regular", &[]));
        let mut cut = font.to_vec();
        cut.truncate(cut.len() / 3);
        let _ = is_same_face("Inter-Regular", &cut);
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
