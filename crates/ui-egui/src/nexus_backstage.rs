//! NexusOffice: the File (Tệp) page of the WPS layout ([`crate::nexus_wps`]), a light page that
//! replaces the document while it is open, as in Nexus Docs and Nexus Sheet. On the left the
//! pages (Information, New, Open, Export) and the actions (Save, Save As, Print, Close); on the
//! right the chosen page. Every action is a registry command, so the page adds no behaviour of its
//! own. Escape, or the back arrow, returns to the document.

use egui::{Align2, Color32, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, icons};

/// The pages of the File page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    /// The document: where it is, its pages and size, its security.
    Info,
    /// New PDFs: blank, from a file, from images, from the clipboard, Combine.
    New,
    /// Open: Browse… and the recent files.
    Open,
    /// Export to Word, PowerPoint, images, text, HTML, RTF.
    Export,
}

const NAV_W: f32 = 230.0;

/// Draws the File page in the window's remaining space.
pub fn show(app: &mut PdfCraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) && !app.modal_open() {
        app.backstage = None;
        return;
    }
    let Some(page) = app.backstage else { return };
    let mut run: Option<&'static str> = None;
    let mut go = None;
    egui::Panel::left("wps_backstage_nav")
        .resizable(false)
        .exact_size(NAV_W)
        .frame(
            egui::Frame::NONE
                .fill(t.titlebar)
                .inner_margin(egui::Margin { left: 10, right: 10, top: 12, bottom: 12 })
                .stroke(Stroke::new(1.0, t.divider)),
        )
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if nav_item(ui, &t, "chevron-left", tl!("Back to the document"), false).clicked() {
                go = Some(None);
            }
            ui.add_space(8.0);
            let has_doc = app.active.is_some();
            for (p, icon, label) in [
                (Page::Info, "info", "Information"),
                (Page::New, "file-plus-2", "New"),
                (Page::Open, "folder-open", "Open"),
                (Page::Export, "file-output", "Export"),
            ] {
                if (has_doc || matches!(p, Page::New | Page::Open)) && nav_item(ui, &t, icon, tl!(label), page == p).clicked() {
                    go = Some(Some(p));
                }
            }
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            for (id, icon, label) in [
                ("file.save", "save", "Save"),
                ("file.save_as", "save", "Save as…"),
                ("print.dialog", "printer", "Print…"),
                ("file.close", "x", "Close file"),
            ] {
                let enabled = pdfcraft_engine::commands::command(id).is_some_and(|s| app.command_enabled(s));
                if ui.add_enabled_ui(enabled, |ui| nav_item(ui, &t, icon, tl!(label), false)).inner.clicked() {
                    run = Some(id);
                }
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                for (id, icon, label) in [("help.about", "info", "About PdfCraft"), ("app.preferences", "settings", "Preferences…")] {
                    if nav_item(ui, &t, icon, tl!(label), false).clicked() {
                        run = Some(id);
                    }
                }
            });
        });
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(t.card).inner_margin(egui::Margin { left: 40, right: 40, top: 28, bottom: 20 })).show(
        ui,
        |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match page {
                Page::Info => info_page(app, ui, &t, &mut run),
                Page::New => new_page(ui, &t, &mut run),
                Page::Open => open_page(app, ui, &t, &mut run),
                Page::Export => export_page(ui, &t, &mut run),
            });
        },
    );
    if let Some(next) = go {
        app.backstage = next;
    }
    if let Some(id) = run {
        app.backstage = None;
        app.execute(id);
    }
}

/// A row of the left column: an icon and a label; the current page on the accent's soft fill.
fn nav_item(ui: &mut Ui, t: &Tokens, icon: &str, label: &str, selected: bool) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    let fill = if selected {
        t.accent_soft
    } else if resp.hovered() {
        t.hover
    } else {
        Color32::TRANSPARENT
    };
    if fill.a() > 0 {
        ui.painter().rect_filled(r, 6.0, fill);
    }
    if selected {
        ui.painter().rect_filled(Rect::from_min_size(r.min + vec2(0.0, 8.0), vec2(3.0, r.height() - 16.0)), 1.5, t.accent);
    }
    let ink = if !ui.is_enabled() {
        t.text_faint
    } else if selected {
        t.accent_text
    } else {
        t.text
    };
    icons::paint(ui, Rect::from_center_size(pos2(r.min.x + 20.0, r.center().y), vec2(16.0, 16.0)), icon, 16.0, ink);
    let font = if selected { theme::semibold(13.0) } else { theme::regular(13.0) };
    ui.painter().text(pos2(r.min.x + 38.0, r.center().y), Align2::LEFT_CENTER, label, font, ink);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label));
    resp
}

fn heading(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.label(egui::RichText::new(text).font(theme::semibold(22.0)).color(t.text));
    ui.add_space(14.0);
}

/// A large button with an icon and two lines (what it does, a short explanation).
fn card(ui: &mut Ui, t: &Tokens, icon: &str, title: &str, detail: &str) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(260.0, 64.0), Sense::click());
    ui.painter().rect(r, 8.0, if resp.hovered() { t.hover } else { t.chrome }, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
    icons::paint(ui, Rect::from_center_size(pos2(r.min.x + 26.0, r.center().y), vec2(22.0, 22.0)), icon, 22.0, t.accent_text);
    ui.painter().text(pos2(r.min.x + 50.0, r.min.y + 22.0), Align2::LEFT_CENTER, title, theme::semibold(13.0), t.text);
    ui.painter().with_clip_rect(r.shrink(4.0)).text(
        pos2(r.min.x + 50.0, r.min.y + 43.0),
        Align2::LEFT_CENTER,
        detail,
        theme::regular(11.5),
        t.text_muted,
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title));
    resp.clicked()
}

fn cards(ui: &mut Ui, t: &Tokens, run: &mut Option<&'static str>, list: &[(&'static str, &str, &str, &str)]) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
        for (id, icon, title, detail) in list {
            if card(ui, t, icon, tl!(title), tl!(detail)) {
                *run = Some(id);
            }
        }
    });
}

fn info_page(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens, run: &mut Option<&'static str>) {
    heading(ui, t, tl!("Information"));
    let Some((_, id)) = app.active_ids() else { return };
    let Some(doc) = app.session.get(id) else { return };
    ui.label(egui::RichText::new(&doc.name).font(theme::semibold(16.0)).color(t.text));
    ui.add_space(10.0);
    let location = doc.path.clone().unwrap_or_else(|| tl!("Not saved yet").to_owned());
    let size = doc.bytes.len();
    let size =
        if size >= 1024 * 1024 { format!("{:.1} MB", size as f64 / (1024.0 * 1024.0)) } else { format!("{:.0} KB", (size as f64 / 1024.0).ceil()) };
    let security = match &doc.read_only_reason {
        Some(why) => why.clone(),
        None if doc.password.is_some() => tl!("Password-protected").to_owned(),
        None => tl!("No restrictions").to_owned(),
    };
    egui::Grid::new("wps_backstage_info").num_columns(2).spacing(vec2(24.0, 8.0)).show(ui, |ui| {
        for (k, v) in [("Location", location), ("Pages", doc.info.pages.len().to_string()), ("File size", size), ("Security", security)] {
            ui.label(egui::RichText::new(tl!(k)).color(t.text_muted));
            ui.label(egui::RichText::new(v).color(t.text));
            ui.end_row();
        }
    });
    ui.add_space(22.0);
    cards(
        ui,
        t,
        run,
        &[
            ("file.properties", "info", "Document properties…", "Title, author, fonts, initial view"),
            ("protect.password", "lock", "Protect using password…", "Ask for a password to open or change it"),
            ("optimize.reduce", "gauge", "Reduce file size…", "Make the file smaller to send"),
            ("file.revert", "rotate-ccw", "Revert", "Go back to the last saved version"),
        ],
    );
}

fn new_page(ui: &mut Ui, t: &Tokens, run: &mut Option<&'static str>) {
    heading(ui, t, tl!("New"));
    cards(
        ui,
        t,
        run,
        &[
            ("create.blank", "file-plus-2", "Blank PDF", "One empty page, in a new tab"),
            ("create.file", "file-input", "From a file…", "A picture or a text file as a PDF"),
            ("create.images", "image-plus", "From images…", "Several pictures, a page each"),
            ("create.clipboard", "clipboard-paste", "From the clipboard", "What you copied, as a PDF"),
            ("page.combine", "files", "Combine files…", "Several files into one PDF"),
        ],
    );
}

fn open_page(app: &mut PdfCraftApp, ui: &mut Ui, t: &Tokens, run: &mut Option<&'static str>) {
    heading(ui, t, tl!("Open"));
    cards(ui, t, run, &[("file.open", "folder-open", "Browse…", "Choose a PDF on this computer")]);
    ui.add_space(20.0);
    ui.label(egui::RichText::new(tl!("Recent files")).font(theme::semibold(14.0)).color(t.text));
    ui.add_space(6.0);
    if app.recent.is_empty() {
        ui.label(egui::RichText::new(tl!("No recent files")).color(t.text_muted));
        return;
    }
    let mut open = None;
    for r in app.recent.iter().take(12) {
        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width().min(720.0), 40.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, 6.0, t.hover);
        }
        icons::paint(ui, Rect::from_center_size(pos2(rect.min.x + 18.0, rect.center().y), vec2(18.0, 18.0)), "file-text", 18.0, theme::APP_COLOR);
        let name = crate::bidi::visual(&r.name).into_owned();
        ui.painter().text(pos2(rect.min.x + 38.0, rect.min.y + 13.0), Align2::LEFT_CENTER, &name, theme::medium(13.0), t.text);
        ui.painter().with_clip_rect(rect).text(
            pos2(rect.min.x + 38.0, rect.min.y + 29.0),
            Align2::LEFT_CENTER,
            &r.path,
            theme::regular(11.0),
            t.text_muted,
        );
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &r.name));
        if resp.clicked() {
            open = Some(r.path.clone());
        }
    }
    if let Some(path) = open {
        app.backstage = None;
        app.open_recent(&path);
    }
}

fn export_page(ui: &mut Ui, t: &Tokens, run: &mut Option<&'static str>) {
    heading(ui, t, tl!("Export"));
    cards(
        ui,
        t,
        run,
        &[
            ("export.docx", "file-text", "Word document…", "Edit the text in Nexus Docs or Word"),
            ("export.pptx", "presentation", "PowerPoint presentation…", "A slide for each page"),
            ("export.image", "image", "Images…", "A PNG or JPEG for each page"),
            ("export.text", "type", "Plain text…", "Only the words"),
            ("export.html", "code-xml", "Web page…", "HTML to open in a browser"),
            ("export.rtf", "file-text", "Rich Text Format…", "RTF for older word processors"),
        ],
    );
}
