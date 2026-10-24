//! NexusOffice: the app embedded in a host window's tab (`PdfCraftApp::embed`), sharing one egui
//! context with other editors.

use egui_kittest::Harness;
use pdfcraft_ui_egui::PdfCraftApp;

/// A 300 pt square page with one line of text in an embedded TrueType font (Inter), so the inline
/// text editor draws it with the document's own font (`pdf-source-*` families).
fn embedded_font_pdf() -> Vec<u8> {
    use pdfcraft_cos::{Dict, Document, Object, Stream, write_incremental};
    use skrifa::MetadataProvider as _;
    let body = "BT /F1 18 Tf 20 220 Td (Stable source text) Tj ET";
    let objs: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Kids [4 0 R] /Count 1 /MediaBox [0 0 300 300] >>".into(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
        "<< /Type /Page /Parent 2 0 R /Contents 5 0 R /Resources << /Font << /F1 3 0 R >> >> >>".into(),
        format!("<< /Length {} >>\nstream\n{body}\nendstream", body.len()),
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
    let mut cos = Document::open(std::sync::Arc::new(out)).unwrap();
    let bytes = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
    let face = skrifa::FontRef::new(bytes).unwrap();
    let metrics = face.metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
    let glyphs = face.glyph_metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
    let units = 1000.0 / f64::from(metrics.units_per_em);
    let program = cos.add(Object::Stream(Stream::flate(Dict::new(), bytes)));
    let mut descriptor = Dict::new();
    descriptor.set(b"FontFile2".to_vec(), Object::Ref(program));
    descriptor.set(b"Ascent".to_vec(), Object::Real(f64::from(metrics.ascent) * units));
    descriptor.set(b"Descent".to_vec(), Object::Real(f64::from(metrics.descent) * units));
    let mut font = Dict::new();
    font.set(b"Type".to_vec(), Object::name("Font"));
    font.set(b"Subtype".to_vec(), Object::name("TrueType"));
    font.set(b"BaseFont".to_vec(), Object::name("Inter-Regular"));
    font.set(b"Encoding".to_vec(), Object::name("WinAnsiEncoding"));
    font.set(b"FirstChar".to_vec(), Object::Int(32));
    font.set(b"LastChar".to_vec(), Object::Int(126));
    font.set(
        b"Widths".to_vec(),
        Object::Array(
            (32..=126u8)
                .map(|c| {
                    let id = face.charmap().map(char::from(c)).unwrap();
                    Object::Real(f64::from(glyphs.advance_width(id).unwrap()) * units)
                })
                .collect(),
        ),
    );
    font.set(b"FontDescriptor".to_vec(), Object::Dict(descriptor));
    cos.update_dict(pdfcraft_cos::ObjRef::new(3, 0), |d| *d = font).unwrap();
    write_incremental(&cos, &Default::default()).unwrap().to_vec()
}

fn embedded(bytes: Vec<u8>) -> Harness<'static, PdfCraftApp> {
    let mut app = PdfCraftApp::new();
    app.set_option("language", "en").unwrap();
    app.embed();
    app.system_clipboard = false;
    app.open_bytes("stable.pdf", None, bytes).unwrap();
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(
        |ui, app: &mut PdfCraftApp| {
            let ctx = ui.ctx().clone();
            app.frame_logic(&ctx);
            app.frame_ui(ui);
        },
        app,
    );
    h.run_steps(6);
    h
}

/// Whether the inline editor draws `text` this frame, in a document font.
fn editor_shows(h: &Harness<'_, PdfCraftApp>, text: &str) -> bool {
    h.output().shapes.iter().any(|s| match &s.shape {
        egui::Shape::Text(t) if t.galley.text() == text => t
            .galley
            .job
            .sections
            .iter()
            .all(|section| matches!(&section.format.font_id.family, egui::FontFamily::Name(name) if name.starts_with("pdf-source-"))),
        _ => false,
    })
}

/// The inline text editor is open (drawing with the document's own font) when another editor in
/// the host window replaces the window's fonts. Back in this tab, its document fonts are installed
/// again with the interface's: the edit stays visible (it vanished, and the app repainted every
/// frame waiting for fonts that never came back).
#[test]
fn the_text_editor_survives_another_editors_fonts() {
    let mut h = embedded(embedded_font_pdf());
    assert!(h.state_mut().execute("edit.edit_text"));
    h.run_steps(2);
    let page = h.state().views[0].page_screen_rect(0).expect("on screen");
    let scale = page.width() / 300.0;
    let point = egui::pos2(page.left() + 40.0 * scale, page.top() + (300.0 - 225.0) * scale);
    h.hover_at(point);
    h.run_steps(1);
    h.drag_at(point);
    h.run_steps(1);
    h.drop_at(point);
    h.run_steps(8);
    let text = h.state().views[0].line_editor.as_ref().expect("the editor opens").text.clone();
    assert!(editor_shows(&h, &text), "the editor draws {text:?} in the document's font");
    h.state_mut().views[0].line_editor.as_mut().unwrap().text.push_str(" edited");
    h.run_steps(2);
    let edited = format!("{text} edited");
    assert!(editor_shows(&h, &edited));
    // Another editor (Nexus Docs) installs a font set of its own: none of this app's families.
    h.ctx.set_fonts(egui::FontDefinitions::default());
    h.run_steps(10);
    // The edit is either on screen again, in the document's font, or applied (the editor closes
    // as it loses the keyboard while the fonts come back); never kept open and invisible.
    let open = h.state().views[0].line_editor.is_some();
    let doc = h.state().session.get(h.state().views[0].id).unwrap();
    let applied = doc.text_blocks(0).iter().any(|b| b.editor_text.contains("edited"));
    assert!(if open { editor_shows(&h, &edited) } else { applied }, "open {open}, applied {applied}");
    assert!(applied || open, "the edit is not lost");
}

/// Two embedded apps in one window number their documents apart: a scroll position kept for one
/// tab's document never moves the other's.
#[test]
fn two_embedded_apps_keep_their_own_documents_and_scroll_positions() {
    let pages = |n: usize| {
        let mut objs: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into()];
        let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
        objs.push(format!("<< /Type /Pages /Kids [{}] /Count {n} /MediaBox [0 0 595 842] >>", kids.join(" ")));
        objs.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into());
        for i in 0..n {
            objs.push(format!("<< /Type /Page /Parent 2 0 R /Contents {} 0 R /Resources << /Font << /F1 3 0 R >> >> >>", 5 + 2 * i));
            let body = format!("BT /F1 24 Tf 72 700 Td (Trang {}) Tj ET", i + 1);
            objs.push(format!("<< /Length {} >>\nstream\n{body}\nendstream", body.len()));
        }
        let mut out = b"%PDF-1.7\n".to_vec();
        let mut offs = Vec::new();
        for (i, o) in objs.iter().enumerate() {
            offs.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
        }
        let x = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
        for o in offs {
            out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{x}\n%%EOF\n", objs.len() + 1).as_bytes());
        out
    };
    let app = |name: &str| {
        let mut a = PdfCraftApp::new();
        a.set_option("language", "en").unwrap();
        a.embed();
        a.system_clipboard = false;
        a.open_bytes(name, None, pages(6)).unwrap();
        a
    };
    let (a, b) = (app("a.pdf"), app("b.pdf"));
    assert_ne!(a.views[0].id, b.views[0].id, "documents of two tabs have different ids");
    // One window (one egui context), showing one app at a time, as the NexusOffice window does.
    let mut h = Harness::builder().with_size(egui::vec2(1024.0, 700.0)).build_ui_state(
        |ui, (a, b, show_b): &mut (PdfCraftApp, PdfCraftApp, bool)| {
            let ctx = ui.ctx().clone();
            let app = if *show_b { b } else { a };
            app.frame_logic(&ctx);
            app.frame_ui(ui);
        },
        (a, b, false),
    );
    h.run_steps(8);
    h.state_mut().0.views[0].go_to_page(4);
    h.run_steps(8);
    assert_eq!(h.state().0.views[0].current, 4, "the first tab scrolled to page 5");
    h.state_mut().2 = true;
    let ctx = h.ctx.clone();
    h.state_mut().1.reactivate(&ctx);
    h.run_steps(8);
    assert_eq!(h.state().1.views[0].current, 0, "the second tab stays on its first page");
}

/// A comment being written is unsaved work: closing asks about it instead of losing it.
#[test]
fn a_comment_being_written_counts_as_unsaved() {
    let mut h = embedded(embedded_font_pdf());
    assert!(h.state().first_dirty().is_none());
    for set in [
        |a: &mut PdfCraftApp| a.views[0].comments.reply = "Đồng ý với đề xuất".into(),
        |a: &mut PdfCraftApp| a.views[0].comments.add_box = "Cần sửa trang 2".into(),
        |a: &mut PdfCraftApp| a.views[0].comments.editing = Some((0, 0, "Sửa lại".into())),
    ] {
        set(h.state_mut());
        assert_eq!(h.state().first_dirty(), Some(0));
        let c = &mut h.state_mut().views[0].comments;
        c.reply.clear();
        c.add_box.clear();
        c.editing = None;
        assert!(h.state().first_dirty().is_none());
    }
    h.run_steps(1);
}
