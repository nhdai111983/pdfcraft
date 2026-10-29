//! NexusOffice (QC-PDF-01/02), in the real shell with the interface in Vietnamese: Add text takes
//! Vietnamese and writes it to the page (in Inter, the default there), and the Watermark and
//! Header & Footer dialogs offer OK for Vietnamese text instead of refusing it.
//! `PDFCRAFT_SHOTS=<dir>` saves screenshots to look at.

use egui::Pos2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_engine::{FontFamily, MarkKind};
use pdfcraft_ui_egui::PdfCraftApp;

/// The warning shown for text the PDF fonts can't draw, in Vietnamese.
const REFUSED: &str = "không thể vẽ";

fn harness() -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).with_step_dt(0.25).build_eframe(move |_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "vi").unwrap();
        app.open_bytes("form.pdf", None, include_bytes!("data/form.pdf").to_vec()).unwrap();
        app.set_option("zoom", "150").unwrap();
        app
    });
    for _ in 0..60 {
        h.run_steps(2);
        if !h.state().render_pending() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h
}

fn at(h: &Harness<'static, PdfCraftApp>, x: f32, y: f32) -> Pos2 {
    let r = h.state().views[0].page_screen_rect(0).expect("on screen");
    let k = r.width() / 300.0;
    r.min + egui::vec2(x * k, y * k)
}

fn click(h: &mut Harness<'static, PdfCraftApp>, p: Pos2) {
    h.hover_at(p);
    h.run_steps(1);
    h.drag_at(p);
    h.run_steps(1);
    h.drop_at(p);
    h.run_steps(3);
}

fn shot(h: &mut Harness<'static, PdfCraftApp>, name: &str) {
    if let Ok(dir) = std::env::var("PDFCRAFT_SHOTS") {
        h.render().unwrap().save(format!("{dir}/{name}.png")).unwrap();
    }
}

#[test]
fn vietnamese_add_text_is_written_in_inter() {
    let mut h = harness();
    assert!(h.state_mut().execute("edit.text"));
    h.run_steps(2);
    let p = at(&h, 40.0, 300.0);
    click(&mut h, p);
    h.event(egui::Event::Text("Đã duyệt — Nguyễn Văn Đức".into()));
    h.run_steps(3);
    assert!(h.query_by_label_contains(REFUSED).is_none(), "no refusal for Vietnamese");
    shot(&mut h, "vi-add-text");
    h.get_by_label("Đã thêm văn bản xong").click();
    h.run_steps(3);
    let s = h.state();
    let added = s.session.get(s.views[0].id).unwrap().added().to_vec();
    let texts: Vec<_> = added
        .iter()
        .filter_map(|a| match &a.content {
            pdfcraft_engine::AddedContent::Text(t) => Some((t.text.clone(), t.family)),
            _ => None,
        })
        .collect();
    assert_eq!(texts, [("Đã duyệt — Nguyễn Văn Đức".to_string(), FontFamily::Inter)]);
}

#[test]
fn vietnamese_watermark_and_header_dialogs_offer_ok() {
    let mut h = harness();
    assert!(h.state_mut().execute("edit.watermark"));
    h.state_mut().marks_draft.wm.text = "TÀI LIỆU MẬT — Đắk Lắk".into();
    h.run_steps(3);
    assert_eq!(h.state().marks_draft.wm.family, FontFamily::Inter, "Vietnamese starts in Inter");
    assert!(h.query_by_label_contains(REFUSED).is_none(), "no refusal for the watermark");
    shot(&mut h, "vi-watermark");
    h.get_by_label("OK").click();
    h.run_steps(3);
    let id = h.state().views[0].id;
    assert!(h.state().session.get(id).unwrap().marks().contains(&MarkKind::Watermark), "the watermark was applied");

    assert!(h.state_mut().execute("edit.header_footer"));
    h.state_mut().marks_draft.hf.text[1] = "Công ty Ánh Dương".into();
    h.run_steps(3);
    assert!(h.query_by_label_contains(REFUSED).is_none(), "no refusal for the header");
    shot(&mut h, "vi-header-footer");
    h.get_by_label("OK").click();
    h.run_steps(3);
    assert!(h.state().session.get(id).unwrap().marks().contains(&MarkKind::HeaderFooter), "the header was applied");
}
