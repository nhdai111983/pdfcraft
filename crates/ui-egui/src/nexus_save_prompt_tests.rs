//! NexusOffice: "Lưu thay đổi?" as a tab of the NexusOffice window ([`crate::nexus_save_prompt`]):
//! the title, the question naming the document, Lưu · Không lưu · Hủy right-aligned in that order,
//! the keyboard on Lưu, Enter saves and Escape cancels.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;

use crate::PdfCraftApp;
use crate::nexus_save_prompt::{BUTTON_SIZE, PADDING, WIDTH};

type H = egui_kittest::Harness<'static, PdfCraftApp>;

const NAME: &str = "Hợp đồng.pdf";

/// A one-page PDF that says "Trang 1".
fn fixture() -> Vec<u8> {
    let body = "BT /F1 24 Tf 72 700 Td (Trang 1) Tj ET";
    let objs: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Kids [4 0 R] /Count 1 /MediaBox [0 0 595 842] >>".into(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
        "<< /Type /Page /Parent 2 0 R /Contents 5 0 R /Resources << /Font << /F1 3 0 R >> >> >>".into(),
        format!("<< /Length {} >>\nstream\n{body}\nendstream", body.len()),
    ];
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
}

/// An embedded Vietnamese window with `NAME` (on disk) open and changed, asking "Lưu thay đổi?".
fn asking(tag: &str, embedded: bool) -> (H, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("nexus-pdf-save-prompt-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(NAME);
    std::fs::write(&path, fixture()).unwrap();
    let mut a = PdfCraftApp::new();
    a.wps = true;
    a.embedded = embedded;
    a.system_clipboard = false;
    a.run_inline = true;
    a.set_option("language", "vi").unwrap();
    a.open_bytes(NAME, Some(path.to_string_lossy().into_owned()), fixture()).unwrap();
    assert!(a.apply_edit(pdfcraft_engine::Edit::RotatePages { pages: vec![0], degrees: 90 }));
    a.request_close_tab(0);
    assert!(a.close_request.is_some());
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_max_steps(64).build_ui_state(
        |ui, app: &mut PdfCraftApp| {
            let ctx = ui.ctx().clone();
            app.frame_logic(&ctx);
            app.frame_ui(ui);
        },
        a,
    );
    h.run_steps(8);
    (h, path)
}

/// The prompt: a dialog named by its title.
fn title(h: &H) -> egui::Rect {
    h.get_by_role_and_label(Role::Dialog, "Lưu thay đổi?").rect()
}

/// The prompt's button `label` (the tool row above may have a button of the same name).
fn button(h: &H, label: &str) -> egui::Rect {
    let card = title(h);
    let found: Vec<_> = h.query_all_by_role_and_label(Role::Button, label).filter(|b| card.contains_rect(b.rect())).collect();
    assert_eq!(found.len(), 1, "one {label} in the prompt");
    found[0].rect()
}

fn focused(h: &H, label: &str) -> bool {
    let card = title(h);
    h.query_all_by_role_and_label(Role::Button, label).any(|b| card.contains_rect(b.rect()) && b.is_focused())
}

fn still_open(h: &H) -> bool {
    !h.state().views.is_empty()
}

#[test]
fn embedded_prompt_reads_like_every_nexus_app() {
    let (h, path) = asking("look", true);
    title(&h);
    h.get_by_label(&format!("Lưu thay đổi của “{NAME}” trước khi đóng?"));
    let (save, dont, cancel) = (button(&h, "Lưu"), button(&h, "Không lưu"), button(&h, "Hủy"));
    assert!(save.max.x <= dont.min.x && dont.max.x <= cancel.min.x, "Lưu, Không lưu, Hủy: {save:?} {dont:?} {cancel:?}");
    assert!((save.center().y - cancel.center().y).abs() < 0.5, "one row");
    for b in [save, dont, cancel] {
        assert!(b.width() >= BUTTON_SIZE.x - 0.5 && b.height() >= BUTTON_SIZE.y - 0.5, "{b:?}");
    }
    let card = title(&h);
    assert!((card.width() - WIDTH).abs() < 3.0, "{card:?}");
    assert!((cancel.max.x - (card.max.x - f32::from(PADDING))).abs() < 1.5, "right-aligned: {cancel:?} {card:?}");
    assert!(focused(&h, "Lưu"), "the keyboard starts on Lưu");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn embedded_prompt_enter_saves_and_escape_cancels() {
    for (key, saves) in [(egui::Key::Enter, true), (egui::Key::Escape, false)] {
        let (mut h, path) = asking(if saves { "enter" } else { "escape" }, true);
        let before = std::fs::read(&path).unwrap();
        h.key_press(key);
        h.run_steps(8);
        assert!(h.state().close_request.is_none(), "{key:?}: answered");
        assert_eq!(std::fs::read(&path).unwrap() != before, saves, "{key:?}: file written");
        assert_eq!(still_open(&h), !saves, "{key:?}: the document closes only after Lưu");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

/// Enter answers with the focused button: Hủy focused and Enter keeps the document.
#[test]
fn enter_on_a_focused_cancel_cancels() {
    let (mut h, path) = asking("focus", true);
    let before = std::fs::read(&path).unwrap();
    let card = title(&h);
    h.query_all_by_role_and_label(Role::Button, "Hủy").find(|b| card.contains_rect(b.rect())).unwrap().focus();
    h.run_steps(2);
    assert!(focused(&h, "Hủy"));
    h.key_press(egui::Key::Enter);
    h.run_steps(6);
    assert!(h.state().close_request.is_none());
    assert!(still_open(&h), "Hủy keeps the document");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn embedded_prompt_dont_save_closes_without_writing() {
    let (mut h, path) = asking("dont", true);
    let before = std::fs::read(&path).unwrap();
    let card = title(&h);
    h.query_all_by_role_and_label(Role::Button, "Không lưu").find(|b| card.contains_rect(b.rect())).unwrap().click();
    h.run_steps(6);
    assert!(h.state().close_request.is_none());
    assert!(!still_open(&h), "closed");
    assert_eq!(std::fs::read(&path).unwrap(), before, "nothing written");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// Stand-alone keeps upstream's prompt.
#[test]
fn stand_alone_prompt_is_upstreams() {
    let (h, path) = asking("standalone", false);
    assert!(h.query_by_label("Lưu thay đổi?").is_none());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

/// Tab and Shift+Tab go through the three buttons in the order they read (the dim and the card
/// never take the focus): Lưu → Không lưu → Hủy → Không lưu.
#[test]
fn tab_cycles_the_buttons_in_reading_order() {
    let (mut h, path) = asking("tab", true);
    let focused = |h: &H, l: &str| {
        let card = title(h);
        h.query_all_by_role_and_label(Role::Button, l).any(|b| card.contains_rect(b.rect()) && b.is_focused())
    };
    assert!(focused(&h, "Lưu"));
    for (shift, want) in [(false, "Không lưu"), (false, "Hủy"), (true, "Không lưu")] {
        let m = if shift { egui::Modifiers::SHIFT } else { egui::Modifiers::NONE };
        h.key_press_modifiers(m, egui::Key::Tab);
        h.run_steps(2);
        assert!(focused(&h, want), "Tab shift={shift}: {want} has the focus");
    }
    assert!(h.state().close_request.is_some(), "still asking");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
