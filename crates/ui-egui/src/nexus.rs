//! NexusOffices branding for the egui front end, kept in one file so upstream's files change as
//! little as possible (the fork's `nexus/BRANDING_PATCH.md` lists every upstream file touched).
//!
//! Interface labels that name upstream's product go through the catalog lookup, which brands
//! them ([`crate::i18n::brand`]); this module holds what the lookup can't express: the Nexus PDF
//! mark (drawn in code: no image file, nothing of ArtCraft's), the About page's Nexus lines and
//! the home screen's card. The names themselves live in [`pdfcraft_engine::nexus`].

use egui::{Color32, CornerRadius, Pos2, Rect, Response, RichText, Sense, Stroke, vec2};
use pdfcraft_engine::nexus::{APP_NAME, ATTRIBUTION, COPYRIGHT, UPSTREAM_COPYRIGHT};

use crate::theme::{self, Tokens};
use crate::widgets;

/// The mark's colours: Nexus PDF red, paper, and the amber of the NexusOffices nodes.
const RED: Color32 = Color32::from_rgb(0xB9, 0x1C, 0x1C);
const PAPER: Color32 = Color32::from_rgb(0xF8, 0xFA, 0xFC);
const AMBER: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);

/// `en` or `vi` by the interface language: the text NexusOffices adds is written in both, and is
/// deliberately not a catalog key (so it is neither branded nor asked of upstream's catalogs).
pub fn text<'a>(en: &'a str, vi: &'a str) -> &'a str {
    if crate::i18n::current().code() == "vi" { vi } else { en }
}

/// The Nexus PDF mark, `size` points square: a red tile with a page and a connected "N" (an
/// original geometric mark by Sonnytech, the same drawing as `nexus/assets/nexus-pdf.svg`).
pub fn mark(ui: &mut egui::Ui, size: f32) -> Response {
    let size = if size.is_finite() { size.clamp(1.0, 512.0) } else { 16.0 };
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Image, true, APP_NAME));
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
    ui.label(
        RichText::new(text("Part of the NexusOffices office suite by Sonnytech.", "Thuộc bộ ứng dụng văn phòng NexusOffices của Sonnytech."))
            .color(t.text_muted),
    );
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
                    "Part of the NexusOffices office suite by Sonnytech. Free and open source.",
                    "Thuộc bộ ứng dụng văn phòng NexusOffices của Sonnytech. Miễn phí, mã nguồn mở.",
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The updater only ever asks the fork for Nexus PDF releases.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn updates_come_from_the_fork() {
        assert_eq!(UPDATE_APP.releases_url(), crate::platform::updates::RELEASES_PAGE);
        assert_eq!(UPDATE_APP.repo_url(), pdfcraft_engine::nexus::SOURCE_URL);
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
