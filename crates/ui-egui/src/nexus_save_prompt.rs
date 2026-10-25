//! NexusOffice: the "Lưu thay đổi?" prompt every NexusOffice app shows when it runs as a tab of
//! the NexusOffice window (`embedded`). Nexus Docs, Sheet, Slide and PDF show the same prompt
//! (Tech Lead ruling 2026-10-25): the title "Lưu thay đổi?", one sentence naming the document,
//! and the buttons Lưu (primary, filled with the accent), Không lưu and Hủy, right-aligned in
//! that order. Enter saves, Escape cancels, the keyboard starts on Lưu, and the prompt is modal
//! over its tab: the tab is dimmed and nothing under the prompt takes a click or a key.
//!
//! The metrics are NexusOffices `DESIGN_SYSTEM.md` §3 (dialog corners 12 px, control corners
//! 6 px, the 4 px grid, dialog title 22, interface text 13), and so are its colours (§2.4: the
//! white surface, the border, the text): the card looks the same in every app. This file is the
//! same, byte for byte, in the four forks; each app passes its accent and title font in a [`Look`].

use egui::accesskit::Role;
use egui::{
    Align, Area, Color32, CornerRadius, FontId, Frame, Id, Key, Label, Layout, Margin, Modifiers, Order, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, UiBuilder, Vec2, pos2, vec2,
};

/// Dialog corners (DESIGN_SYSTEM.md §3: "hộp thoại 12 px").
pub const DIALOG_RADIUS: u8 = 12;
/// Button corners (DESIGN_SYSTEM.md §3: "điều khiển 6 px").
pub const BUTTON_RADIUS: u8 = 6;
/// Space between the card's edge and its content, on the 4 px grid.
pub const PADDING: i8 = 20;
/// The smallest button; a longer label makes it wider.
pub const BUTTON_SIZE: Vec2 = vec2(88.0, 32.0);
/// Space between two buttons.
pub const BUTTON_GAP: f32 = 8.0;
/// The title's size (DESIGN_SYSTEM.md §3: "tiêu đề hộp thoại 22").
pub const TITLE_SIZE: f32 = 22.0;
/// The question's and the buttons' size (DESIGN_SYSTEM.md §3: "cỡ giao diện 13").
pub const TEXT_SIZE: f32 = 13.0;
/// The card's width.
pub const WIDTH: f32 = 420.0;

/// The card's colours (DESIGN_SYSTEM.md §2.4), light and dark: the surface, its border, the text
/// and the second line's text.
pub const LIGHT: Palette = Palette {
    surface: Color32::WHITE,
    border: Color32::from_rgb(0xE2, 0xE8, 0xF0),
    text: Color32::from_rgb(0x0F, 0x17, 0x2A),
    text_dim: Color32::from_rgb(0x47, 0x55, 0x69),
    backdrop: Color32::from_black_alpha(56),
};
pub const DARK: Palette = Palette {
    surface: Color32::from_rgb(0x1E, 0x29, 0x3B),
    border: Color32::from_rgb(0x33, 0x41, 0x55),
    text: Color32::from_rgb(0xF1, 0xF5, 0xF9),
    text_dim: Color32::from_rgb(0x94, 0xA3, 0xB8),
    backdrop: Color32::from_black_alpha(110),
};

/// The card's colours, the same in every app ([`LIGHT`], [`DARK`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub surface: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    /// The dim over the tab.
    pub backdrop: Color32,
}

/// What the user answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Save,
    DontSave,
    Cancel,
}

/// The prompt's words, in Vietnamese or (any other interface language) English.
#[derive(Clone, Copy, Debug)]
pub struct Words {
    pub title: &'static str,
    pub save: &'static str,
    pub dont_save: &'static str,
    pub cancel: &'static str,
    /// The optional second line: what happens without saving.
    pub lost: &'static str,
}

pub const VI: Words =
    Words {
        title: "Lưu thay đổi?", save: "Lưu", dont_save: "Không lưu", cancel: "Hủy", lost: "Nếu không lưu, các thay đổi sẽ mất."
    };
pub const EN: Words =
    Words { title: "Save changes?", save: "Save", dont_save: "Don't Save", cancel: "Cancel", lost: "If you don't save, your changes will be lost." };

pub fn words(vi: bool) -> Words {
    if vi { VI } else { EN }
}

/// "Lưu thay đổi của “<name>” trước khi đóng?" A long name (a web address has no limit) is cut in
/// the middle so the prompt stays inside the window.
pub fn question(vi: bool, name: &str) -> String {
    let name = shorten_middle(name, 80);
    if vi { format!("Lưu thay đổi của “{name}” trước khi đóng?") } else { format!("Save changes to “{name}” before closing?") }
}

/// `name` cut to at most `max` characters by replacing its middle with "…". Counts `char`s, so it
/// never splits one.
pub fn shorten_middle(name: &str, max: usize) -> String {
    let count = name.chars().count();
    if count <= max {
        return name.to_string();
    }
    let tail = max / 4;
    let head: String = name.chars().take(max.saturating_sub(tail + 1)).collect();
    let end: String = name.chars().skip(count.saturating_sub(tail)).collect();
    format!("{head}…{end}")
}

/// What each app brings: whether its theme is dark, its accent and its bold face.
#[derive(Clone, Debug)]
pub struct Look {
    /// The dark card ([`DARK`]) instead of the light one ([`LIGHT`]).
    pub dark: bool,
    /// Lưu's fill (the app's or the user's accent) and the focus ring.
    pub accent: Color32,
    /// Lưu's label, readable on `accent`.
    pub on_accent: Color32,
    /// Why the last save failed.
    pub error: Color32,
    /// The title's font at [`TITLE_SIZE`]: the app's semibold face.
    pub title_font: FontId,
    /// The app has no semibold face here (`title_font` is its regular one): the title is drawn
    /// twice, a fraction of a point apart, so it reads as bold in every app all the same.
    pub title_fake_bold: bool,
}

/// What the prompt says.
#[derive(Clone, Debug)]
pub struct Prompt<'a> {
    pub words: Words,
    /// The question ([`question`]).
    pub question: &'a str,
    /// A second line, only when needed (for example [`Words::lost`]).
    pub note: Option<&'a str>,
    /// Why the last save failed, when the prompt came back after a failed save.
    pub error: Option<&'a str>,
}

/// Draw the prompt over `over` (the tab's rectangle; the whole window when it is empty) and return
/// the answer once there is one. Call it every frame while the question is open.
pub fn show(ctx: &egui::Context, id: Id, over: Rect, prompt: &Prompt<'_>, look: &Look) -> Option<Answer> {
    let over = if over.is_positive() && over.is_finite() { over } else { ctx.content_rect() };
    let pal = if look.dark { DARK } else { LIGHT };
    let size_id = id.with("size");
    let last = ctx.data(|d| d.get_temp::<Vec2>(size_id)).unwrap_or(vec2(WIDTH, 170.0));
    let card = Rect::from_center_size(over.center(), last);
    // A tab smaller than the card: keep the title and the buttons' top inside it.
    let card = card.translate((over.min - card.min).max(Vec2::ZERO));
    let area = Area::new(id).order(Order::Foreground).fixed_pos(over.min).constrain(false);
    // Modal: nothing under this layer takes a click, and the keyboard focus stays in it.
    ctx.memory_mut(|m| m.set_modal_layer(area.layer()));
    let words = prompt.words;
    // Tab and Shift+Tab are the prompt's: it moves the focus itself, in reading order and around,
    // so egui's own Tab never hands it to a widget of the app or the host window.
    let (tab, back_tab) = ctx.input(|i| {
        let tab = i.key_pressed(Key::Tab);
        (tab && i.modifiers.is_none(), tab && i.modifiers.shift_only())
    });
    let was = focused(ctx, id);
    if tab || back_tab {
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Tab);
            i.consume_key(Modifiers::SHIFT, Key::Tab);
        });
    }
    // Clicks and drags, but never the keyboard focus: Tab goes only through the three buttons.
    let blocks = Sense::CLICK | Sense::DRAG;
    let shown = area.show(ctx, |ui| {
        // The dim over the tab, taking the clicks that would reach the document.
        let _ = ui.allocate_rect(over, blocks);
        ui.painter().rect_filled(over, CornerRadius::ZERO, pal.backdrop);
        ui.scope_builder(UiBuilder::new().id_salt("card").max_rect(card).sense(blocks), |ui| {
            Frame::NONE
                .fill(pal.surface)
                .stroke(Stroke::new(1.0, pal.border))
                .corner_radius(CornerRadius::same(DIALOG_RADIUS))
                .inner_margin(Margin::same(PADDING))
                .shadow(ui.visuals().popup_shadow)
                .show(ui, |ui| {
                    let inner = WIDTH - 2.0 * f32::from(PADDING);
                    ui.set_width(inner);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    title(ui, words.title, look, pal.text, inner);
                    ui.add_space(12.0);
                    ui.add(Label::new(RichText::new(prompt.question).size(TEXT_SIZE).color(pal.text)).wrap());
                    if let Some(note) = prompt.note {
                        ui.add_space(4.0);
                        ui.add(Label::new(RichText::new(note).size(TEXT_SIZE).color(pal.text_dim)).wrap());
                    }
                    if let Some(error) = prompt.error.filter(|e| !e.is_empty()) {
                        ui.add_space(8.0);
                        ui.add(Label::new(RichText::new(error).size(TEXT_SIZE).color(look.error)).wrap());
                    }
                    ui.add_space(24.0);
                    // Right-aligned, but laid out left to right so that Tab follows what the eye
                    // reads: Lưu, Không lưu, Hủy.
                    let labels = [words.save, words.dont_save, words.cancel];
                    let pad = ui.spacing().button_padding.x * 2.0;
                    let widths = labels.map(|l| {
                        let w = ui.painter().layout_no_wrap(l.to_string(), FontId::proportional(TEXT_SIZE), pal.text).size().x;
                        (w + pad).max(BUTTON_SIZE.x)
                    });
                    let row = widths.iter().sum::<f32>() + 2.0 * BUTTON_GAP;
                    ui.allocate_ui_with_layout(vec2(inner, BUTTON_SIZE.y), Layout::left_to_right(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = BUTTON_GAP;
                        let lead = (inner - row).max(0.0);
                        if lead > 0.0 {
                            ui.add_space(lead);
                        }
                        let save = button(ui, words.save, true, look, &pal);
                        let dont_save = button(ui, words.dont_save, false, look, &pal);
                        let cancel = button(ui, words.cancel, false, look, &pal);
                        [(save, Answer::Save), (dont_save, Answer::DontSave), (cancel, Answer::Cancel)]
                    })
                    .inner
                })
        })
    });
    let card_id = shown.inner.response.id;
    let card_rect = shown.inner.inner.response.rect;
    // Screen readers (and kittest) meet a modal dialog named by its title, where the card is.
    let _ = ctx.accesskit_node_builder(card_id, |node| {
        node.set_role(Role::Dialog);
        node.set_label(words.title);
        node.set_modal();
        node.set_bounds(egui::accesskit::Rect {
            x0: card_rect.min.x.into(),
            y0: card_rect.min.y.into(),
            x1: card_rect.max.x.into(),
            y1: card_rect.max.y.into(),
        });
    });
    let buttons = shown.inner.inner.inner;
    let size = card_rect.size();
    if (size - last).length() > 0.5 {
        ctx.data_mut(|d| d.insert_temp(size_id, size));
        ctx.request_repaint();
    }
    // The keyboard starts on Lưu and comes back to it from anywhere outside the three buttons;
    // Tab and Shift+Tab step through them (`buttons` is in reading order).
    let here = buttons.iter().position(|(b, _)| b.has_focus()).or_else(|| was.and_then(|w| buttons.iter().position(|(_, a)| *a == w)));
    let step = match (tab, back_tab, here) {
        (true, _, Some(i)) => Some((i + 1) % buttons.len()),
        (_, true, Some(i)) => Some((i + buttons.len() - 1) % buttons.len()),
        (true, _, None) => Some(0),
        (_, true, None) => Some(buttons.len() - 1),
        _ => None,
    };
    let focused_answer = match step.and_then(|n| buttons.get(n)) {
        Some((b, a)) => {
            b.request_focus();
            Some(*a)
        }
        None => buttons.iter().find(|(b, _)| b.has_focus()).map(|(_, a)| *a),
    };
    if focused_answer.is_none()
        && let Some((save, _)) = buttons.first()
    {
        save.request_focus();
    }
    ctx.data_mut(|d| d.insert_temp(id.with("focus"), focused_answer));
    let focused = focused_answer.is_some();
    let (escape, enter) = ctx.input(|i| (i.key_pressed(Key::Escape), i.key_pressed(Key::Enter)));
    let answer = if escape {
        Some(Answer::Cancel)
    } else if let Some((_, a)) = buttons.iter().find(|(b, _)| b.clicked()) {
        // Enter or Space on the focused button clicks it.
        Some(*a)
    } else if enter && !focused {
        Some(Answer::Save)
    } else {
        None
    };
    if answer.is_some() {
        // The keys belong to the prompt, not to whatever reads them after it this frame.
        ctx.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::Enter);
            i.consume_key(Modifiers::NONE, Key::Escape);
        });
    }
    answer
}

/// The button that has the keyboard focus in the prompt `id` ([`show`]'s last frame): what Enter
/// means for an app that reads the prompt's keys before it draws the prompt.
pub fn focused(ctx: &egui::Context, id: Id) -> Option<Answer> {
    ctx.data(|d| d.get_temp::<Option<Answer>>(id.with("focus"))).flatten()
}

/// The title, painted (the dialog carries it as its accessible name, so it isn't a second label).
fn title(ui: &mut egui::Ui, text: &str, look: &Look, color: Color32, width: f32) {
    let galley = ui.painter().layout(text.to_string(), look.title_font.clone(), color, width);
    let (rect, _) = ui.allocate_exact_size(vec2(width, galley.size().y), Sense::hover());
    let at = pos2(rect.min.x, rect.min.y);
    if look.title_fake_bold {
        ui.painter().galley(at + vec2(0.6, 0.0), galley.clone(), color);
    }
    ui.painter().galley(at, galley, color);
}

fn button(ui: &mut egui::Ui, text: &str, primary: bool, look: &Look, pal: &Palette) -> Response {
    let (fill, stroke, color) =
        if primary { (look.accent, Stroke::NONE, look.on_accent) } else { (pal.surface, Stroke::new(1.0, pal.border), pal.text) };
    let r = ui.add(
        egui::Button::new(RichText::new(text).size(TEXT_SIZE).color(color))
            .fill(fill)
            .stroke(stroke)
            .corner_radius(CornerRadius::same(BUTTON_RADIUS))
            .min_size(BUTTON_SIZE),
    );
    let radius = CornerRadius::same(BUTTON_RADIUS);
    if r.is_pointer_button_down_on() {
        ui.painter().rect_filled(r.rect, radius, Color32::from_black_alpha(36));
    } else if r.hovered() {
        ui.painter().rect_filled(r.rect, radius, Color32::from_black_alpha(18));
    }
    if r.has_focus() {
        ui.painter().rect_stroke(r.rect.expand(2.0), CornerRadius::same(BUTTON_RADIUS + 2), Stroke::new(2.0, look.accent), StrokeKind::Outside);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_names_the_document_and_long_names_are_cut() {
        assert_eq!(question(true, "Báo cáo.docx"), "Lưu thay đổi của “Báo cáo.docx” trước khi đóng?");
        let long = "đ".repeat(200);
        let q = question(true, &long);
        assert!(q.chars().count() < 130, "{q}");
        assert!(q.contains('…'));
        assert_eq!(shorten_middle("", 0), "");
        assert_eq!(shorten_middle("abc", 0), "…");
        assert_eq!(shorten_middle("abc", 3), "abc");
    }
}
