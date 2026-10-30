//! NexusOffice branding for the egui front end, kept in one file so upstream's files change as
//! little as possible (the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched).
//!
//! Interface labels that name upstream's product go through the catalog lookup, which brands
//! them ([`crate::i18n::brand`]); this module holds what the lookup can't express: the Nexus PDF
//! mark (drawn in code: no image file, nothing of ArtCraft's), the About page's Nexus lines and
//! the home screen's card. The names themselves live in [`pdfcraft_engine::nexus`].

use egui::{Color32, CornerRadius, Pos2, Rect, Response, RichText, Sense, Stroke, vec2};
use pdfcraft_engine::nexus::{ABOUT_SUITE, APP_NAME, ATTRIBUTION, COPYRIGHT, UPSTREAM_COPYRIGHT};

use crate::theme::{self, Tokens};
use crate::widgets;

/// The mark's colours: Nexus PDF red, paper, and the amber of the NexusOffice nodes.
const RED: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
const PAPER: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
const AMBER: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);

/// `en` or `vi` by the interface language: the text NexusOffice adds is written in both, and is
/// deliberately not a catalog key (so it is neither branded nor asked of upstream's catalogs).
pub fn text<'a>(en: &'a str, vi: &'a str) -> &'a str {
    if crate::i18n::current().code() == "vi" { vi } else { en }
}

/// The Nexus PDF mark, `size` points square: a red tile with a page and a connected "N" (an
/// original geometric mark by SonnyTech, the same drawing as `nexus/assets/nexus-pdf.svg`).
pub fn mark(ui: &mut egui::Ui, size: f32) -> Response {
    let size = if size.is_finite() { size.clamp(1.0, 512.0) } else { 16.0 };
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, true, APP_NAME));
    if ui.is_rect_visible(rect) {
        paint_mark(ui.painter(), rect);
    }
    response
}

/// Paint the mark into `rect` (drawn on a 512-unit grid scaled to the rect's width).
pub fn paint_mark(painter: &egui::Painter, rect: Rect) {
    let s = rect.width() / 512.0;
    let at = |x: f32, y: f32| Pos2::new(rect.min.x + x * s, rect.min.y + y * s);
    let radius = |r: f32| CornerRadius::same((r * s).round().clamp(0.0, 255.0) as u8);
    painter.rect_filled(rect, radius(112.0), RED);
    painter.rect_filled(Rect::from_min_max(at(120.0, 88.0), at(392.0, 424.0)), radius(28.0), PAPER);
    painter.add(egui::Shape::line(vec![at(192.0, 344.0), at(192.0, 168.0), at(320.0, 344.0), at(320.0, 168.0)], Stroke::new(36.0 * s, RED)));
    for (x, y) in [(320.0, 168.0), (192.0, 344.0)] {
        painter.circle_filled(at(x, y), 26.0 * s, AMBER);
    }
}

/// About: the lines under the name and version, where upstream showed "Part of" and the ArtCraft
/// wordmark: the suite and publisher, then the attribution and copyright notices in text.
pub fn about_lines(ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(RichText::new(text(ABOUT_SUITE.0, ABOUT_SUITE.1)).color(t.text_muted));
    ui.add_space(6.0);
    ui.label(RichText::new(text(ATTRIBUTION.0, ATTRIBUTION.1)).color(t.text_muted).small());
    ui.label(RichText::new(UPSTREAM_COPYRIGHT).color(t.text_muted).small());
    ui.label(RichText::new(COPYRIGHT).color(t.text_muted).small());
}

/// The home screen's first card, where upstream invited people to the ArtCraft community: the
/// mark, the suite, and the link to the source. Returns the registry command of a link clicked.
/// Its close button sets `closed` (for this run only, as upstream's card).
pub fn home_card(ui: &mut egui::Ui, closed: &mut bool) -> Option<&'static str> {
    let t = Tokens::get(ui.ctx());
    ui.horizontal(|ui| {
        mark(ui, 28.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(APP_NAME).font(theme::semibold(15.0)));
            ui.label(
                RichText::new(text(
                    "Part of the NexusOffice suite by SonnyTech. Free and open source.",
                    "Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech. Miễn phí, mã nguồn mở.",
                ))
                .color(t.text_muted),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if crate::icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                *closed = true;
            }
        });
    });
    ui.add_space(8.0);
    widgets::community_links(ui)
}

/// The app Help ▸ Check for updates asks about (`craft_update`): Nexus PDF in the fork's
/// releases, never upstream's `suite::PDFCRAFT`, so the updater can only offer, download or
/// install a Nexus PDF release. The id is the installed program and Linux package name
/// ([`pdfcraft_engine::nexus::APP_BIN`]); the display name is [`APP_NAME`] (the Windows default
/// install folder `Programs\Nexus PDF` and the HTTP User-Agent `Nexus PDF/Update`).
#[cfg(not(target_arch = "wasm32"))]
pub const UPDATE_APP: craft_update::App = craft_update::App {
    id: pdfcraft_engine::nexus::APP_BIN,
    name: APP_NAME,
    tagline: "Reading, organizing and protecting PDFs",
    repo: "nhdai111983/pdfcraft",
    former_ids: &[],
    former_names: &[],
    icon_png: None,
};

/// NexusOffice: Nexus PDF never checks for, downloads or installs updates itself (upstream's
/// `craft_update` start-up check and self-update would ask `storytold/pdfcraft`); new versions
/// arrive through the NexusOffice installer, and Help ▸ Check for updates says so.
pub const UPDATES_THROUGH_INSTALLER: bool = true;

/// What Help ▸ Check for updates says, in the interface language (en or vi), for version `version`.
pub fn updates_message(version: &str) -> String {
    let template = text(
        "Nexus PDF {v} is updated through the NexusOffice installer. New versions arrive with NexusOffice; there is nothing to download here.",
        "Nexus PDF {v} được cập nhật qua trình cài đặt NexusOffice. Phiên bản mới đến cùng NexusOffice; không cần tải gì ở đây.",
    );
    template.replace("{v}", version)
}

/// Preferences › Appearance: the accent colour row — the app colour, the NexusOffice presets and
/// any colour from the picker. Every choice goes through the `accent` option, as the control
/// channel and the command line set it.
pub fn accent_picker(ui: &mut egui::Ui, app: &mut crate::PdfCraftApp) {
    let t = Tokens::get(ui.ctx());
    let current = app.accent;
    let mut chosen: Option<Option<Color32>> = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(text("Accent colour", "Màu chủ đạo"));
        if ui.selectable_label(current.is_none(), text("App colour", "Màu ứng dụng")).clicked() {
            chosen = Some(None);
        }
        for (en, vi, c) in theme::ACCENT_PRESETS {
            let name = text(en, vi);
            let (r, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
            let on = current == Some(c);
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, on, name));
            if ui.is_rect_visible(r) {
                ui.painter().circle_filled(r.center(), 8.0, c);
                if on {
                    ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(2.0, t.text));
                } else if resp.hovered() {
                    ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(1.0, t.border));
                }
            }
            if theme::hand(resp).on_hover_text(name).clicked() {
                chosen = Some(Some(c));
            }
        }
        let mut pick = current.unwrap_or(theme::APP_COLOR);
        let picker = ui.color_edit_button_srgba(&mut pick).on_hover_text(text("Any colour", "Màu bất kỳ"));
        if picker.changed() {
            chosen = Some(Some(Color32::from_rgb(pick.r(), pick.g(), pick.b())));
        }
    });
    ui.label(
        RichText::new(text(
            "Buttons, selected tabs and highlights use it; text on it stays readable.",
            "Dùng cho nút, thẻ đang chọn và điểm nhấn; chữ trên màu luôn đọc rõ.",
        ))
        .color(t.text_muted)
        .small(),
    );
    if let Some(accent) = chosen {
        let value = accent.map_or_else(|| "none".to_owned(), theme::to_hex);
        if let Err(e) = app.set_option("accent", &value) {
            app.notify(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Help ▸ Check for updates names the NexusOffice installer, in English and Vietnamese.
    #[test]
    fn check_for_updates_points_at_the_installer_in_en_and_vi() {
        let previous = crate::i18n::current();
        crate::i18n::set_current(crate::i18n::Lang::from_code("en").unwrap());
        let en = updates_message("1.2.3");
        assert!(en.contains("Nexus PDF 1.2.3") && en.contains("NexusOffice installer"), "{en}");
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        let vi = updates_message("1.2.3");
        assert!(vi.contains("Nexus PDF 1.2.3") && vi.contains("trình cài đặt NexusOffice"), "{vi}");
        crate::i18n::set_current(previous);
    }

    /// The updater only ever asks the fork for Nexus PDF releases.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn updates_come_from_the_fork() {
        assert_eq!(UPDATE_APP.releases_url(), crate::platform::updates::RELEASES_PAGE);
        // NX-BRAND.1: the source moved to sonnytech-vn, but updates stay with the repository's old
        // address (with RELEASES_PAGE) until it is transferred; GitHub then redirects.
        assert_eq!(UPDATE_APP.repo_url(), "https://github.com/nhdai111983/pdfcraft");
        assert!(crate::platform::updates::RELEASES_PAGE.starts_with(&UPDATE_APP.repo_url()));
        assert_eq!((UPDATE_APP.id, UPDATE_APP.name), ("nexus-pdf", "Nexus PDF"));
        assert!(!UPDATE_APP.repo.contains("storytold"));
    }

    #[test]
    fn text_follows_the_interface_language() {
        let previous = crate::i18n::current();
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap());
        assert_eq!(text("Source", "Mã nguồn"), "Mã nguồn");
        crate::i18n::set_current(crate::i18n::Lang::from_code("en").unwrap());
        assert_eq!(text("Source", "Mã nguồn"), "Source");
        crate::i18n::set_current(previous);
    }

    /// The `accent` option (command line, `ui.set`, Preferences) sets, reads back through
    /// `ui.state`, persists and resets the accent colour; junk is refused and changes nothing.
    #[test]
    fn the_accent_option_sets_persists_and_resets() {
        use crate::control::Host as _;
        let indigo = Color32::from_rgb(0x63, 0x66, 0xF1);
        let mut app = crate::PdfCraftApp::new();
        assert_eq!(app.accent, None);
        assert_eq!(app.state()["accent"], serde_json::Value::Null);
        assert!(!serde_json::from_str::<serde_json::Value>(&app.persist()).unwrap().as_object().unwrap().contains_key("accent"));
        app.set_option("accent", "#6366f1").unwrap();
        assert_eq!(app.accent, Some(indigo));
        assert_eq!(app.state()["accent"], "#6366F1");
        for junk in ["purple", "#12345", "#1234567", "#ÿÿÿÿÿÿ", "ÿÿÿÿÿÿ", "12 34 56", "#gggggg", "rgb(1,2,3)"] {
            assert!(app.set_option("accent", junk).is_err(), "{junk:?}");
            assert_eq!(app.accent, Some(indigo), "{junk:?} changed the accent");
        }
        assert!(app.set("accent", "nonsense").is_err(), "the control channel refuses junk too");
        let saved = app.persist();
        assert_eq!(serde_json::from_str::<serde_json::Value>(&saved).unwrap()["accent"], "#6366F1");
        let mut back = crate::PdfCraftApp::new();
        back.restore(&saved);
        assert_eq!(back.accent, Some(indigo), "the accent survives a restart");
        for reset in ["none", "", " default ", "app"] {
            app.set_option("accent", "#10B981").unwrap();
            app.set_option("accent", reset).unwrap();
            assert_eq!(app.accent, None, "{reset:?}");
        }
        assert!(!serde_json::from_str::<serde_json::Value>(&app.persist()).unwrap().as_object().unwrap().contains_key("accent"));
    }

    /// Saved settings are untrusted: a saved accent that isn't `#RRGGBB` reads as the app colour.
    #[test]
    fn a_garbage_saved_accent_reads_as_the_app_colour() {
        for saved in [
            r#"{"accent": 42}"#,
            r##"{"accent": "#zzzzzz"}"##,
            r##"{"accent": "#ÿÿÿÿÿÿ"}"##,
            r#"{"accent": null}"#,
            r#"{"accent": [255, 0, 0]}"#,
            r#"{"accent": {"r": 1}}"#,
            r##"{"accent": "#F43F5E00"}"##,
            r#"{"accent": ""}"#,
            r#"{}"#,
            "not json",
        ] {
            let mut app = crate::PdfCraftApp::new();
            app.accent = Some(Color32::from_rgb(1, 2, 3));
            app.restore(saved);
            let expected = if saved == "not json" { Some(Color32::from_rgb(1, 2, 3)) } else { None };
            assert_eq!(app.accent, expected, "{saved}");
        }
    }

    /// Preferences › Appearance offers the app colour, the six presets and a picker; clicking a
    /// preset sets the accent, and the palette the widgets read follows it; App colour resets it.
    #[test]
    fn preferences_pick_an_accent_colour() {
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;
        let mut app = crate::PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        let mut fonts = false;
        let mut h = Harness::builder().with_size(egui::vec2(720.0, 480.0)).build_ui_state(
            move |ui, app: &mut crate::PdfCraftApp| {
                // The app's fonts (the semibold headings) take effect from the next frame.
                if !fonts {
                    ui.ctx().set_fonts(theme::font_definitions());
                    fonts = true;
                    return;
                }
                // What `logic` does every frame.
                theme::set_accent(ui.ctx(), app.accent);
                let t = Tokens::get(ui.ctx());
                crate::appearance::preferences_section(ui, app, &t);
            },
            app,
        );
        h.run();
        assert!(h.query_by_label("Accent colour").is_some());
        for (en, _, _) in theme::ACCENT_PRESETS {
            assert!(h.query_by_label(en).is_some(), "{en}");
        }
        h.get_by_label("Indigo").click();
        h.run();
        let indigo = Color32::from_rgb(0x63, 0x66, 0xF1);
        assert_eq!(h.state().accent, Some(indigo));
        assert_eq!(Tokens::get(&h.ctx), theme::with_accent(Tokens::for_kind(theme::ThemeKind::Light), indigo));
        if let Ok(path) = std::env::var("PDFCRAFT_ACCENT_SHOT") {
            h.render().unwrap().save(path).unwrap();
        }
        h.get_by_label("App colour").click();
        h.run();
        assert_eq!(h.state().accent, None);
        assert_eq!(Tokens::get(&h.ctx), Tokens::for_kind(theme::ThemeKind::Light));
    }

    /// The mark is drawn in code, labelled with the product name, and survives odd sizes.
    #[test]
    fn the_mark_is_drawn_in_code_and_labelled() {
        let ctx = egui::Context::default();
        for size in [40.0, 0.0, -3.0, f32::NAN, f32::INFINITY, 1e9] {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                let r = mark(ui, size);
                assert!(r.rect.width() >= 1.0 && r.rect.width() <= 512.0, "{size}: {:?}", r.rect);
            });
            out.textures_delta.clear();
        }
        // No image bytes: nothing of docs/brand/ is compiled in any more.
        let source = include_str!("widgets.rs");
        assert!(!source.contains(concat!("docs/", "brand/")), "widgets.rs embeds a brand file");
    }
}
