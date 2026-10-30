//! NexusOffice Vietnamese defaults for page text and page marks.
//!
//! QC-PDF-01/02: the font new page text starts in. With the interface in
//! Vietnamese, Add text and the Header & Footer and Watermark dialogs start in Inter, the bundled
//! face that draws every Vietnamese letter (embedded as a subset), until the user picks a font
//! themselves in this session. Text in a standard font that needs more than WinAnsi is drawn in
//! Inter anyway (the engine decides); this only makes the choice visible from the start.
//!
//! QC-PDF-06/09: in Vietnamese, margins show in centimetres rather than inches, the date formats
//! list day-first ones first, and number fields use a decimal comma (`0,50`, `100,0%`).

use std::sync::atomic::{AtomicBool, Ordering};

use pdfcraft_engine::FontFamily;

/// The user picked a font family this session: their choice stands.
static PICKED: AtomicBool = AtomicBool::new(false);

/// The user chose a family in a font picker.
pub(crate) fn note_picked() {
    PICKED.store(true, Ordering::Relaxed);
}

/// The family to show for `current`: Inter instead of the engine's Helvetica default while the
/// interface is Vietnamese and nobody picked a font.
pub(crate) fn default_family(current: FontFamily) -> FontFamily {
    if current == FontFamily::Helvetica && !PICKED.load(Ordering::Relaxed) && crate::i18n::current().code() == "vi" {
        FontFamily::Inter
    } else {
        current
    }
}

/// The families a picker offers: the standard three, and Inter where the engine embeds it
/// (added text and page marks; Edit text keeps to the standard fonts).
pub(crate) fn families(unicode: bool) -> &'static [FontFamily] {
    if unicode {
        &[FontFamily::Inter, FontFamily::Helvetica, FontFamily::Times, FontFamily::Courier]
    } else {
        &[FontFamily::Helvetica, FontFamily::Times, FontFamily::Courier]
    }
}

/// The picker's label for `f`: Inter says it has Vietnamese.
pub(crate) fn label(f: FontFamily) -> String {
    match f {
        FontFamily::Inter => format!("{} ({})", f.label(), crate::nexus_wps::tx("Vietnamese")),
        other => other.label().to_string(),
    }
}

/// A font-family picker; returns true when the user changed it.
pub(crate) fn picker(ui: &mut egui::Ui, id: &str, family: &mut FontFamily, unicode: bool, width: f32) -> bool {
    let before = *family;
    craft_ui::ComboBox::from_id_salt(id).selected_text(label(*family)).width(width).show_ui(ui, |ui| {
        for f in families(unicode) {
            ui.selectable_value(family, *f, label(*f)).on_hover_text(match f {
                FontFamily::Inter => crate::nexus_wps::tx("Embedded in the PDF; draws every Vietnamese letter."),
                _ => crate::nexus_wps::tx("A standard PDF font; Western European letters only."),
            });
        }
    });
    let changed = *family != before;
    if changed {
        note_picked();
    }
    changed
}

/// A comment date ("2026-10-26 13:39", already local) as the interface writes dates: day first
/// in Vietnamese ("26/10/2026 13:39", QC-PDF-04). Anything else is shown as it is.
pub(crate) fn date(s: &str) -> String {
    let b = s.as_bytes();
    let shape = b.len() == 16
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b' ',
            13 => *c == b':',
            _ => c.is_ascii_digit(),
        });
    match (vi() && shape, s.get(0..4), s.get(5..7), s.get(8..10), s.get(11..16)) {
        (true, Some(y), Some(m), Some(d), Some(t)) => format!("{d}/{m}/{y} {t}"),
        _ => s.to_string(),
    }
}

fn vi() -> bool {
    crate::i18n::current().code() == "vi"
}

/// The unit lengths show in: centimetres instead of inches while the interface is Vietnamese
/// (Vietnam measures in centimetres; inches only come from a US region setting).
pub(crate) fn units(u: crate::units::Unit) -> crate::units::Unit {
    if vi() && u == crate::units::Unit::Inches { crate::units::Unit::Centimetres } else { u }
}

/// The Header & Footer date tokens, in the order the picker lists them: day first in Vietnamese
/// (no English month names there).
pub(crate) fn date_formats(english: &'static [&'static str]) -> &'static [&'static str] {
    const VI: [&str; 10] = [
        "<<dd/mm/yyyy>>",
        "<<d/m/yyyy>>",
        "<<d/m/yy>>",
        "<<dd.mm.yyyy>>",
        "<<d.m.yyyy>>",
        "<<dd.mm.yy>>",
        "<<yyyy-mm-dd>>",
        "<<yyyy/mm/dd>>",
        "<<mm/dd/yyyy>>",
        "<<m/d/yyyy>>",
    ];
    if vi() { &VI } else { english }
}

/// `v` with `decimals`, with a decimal comma in Vietnamese.
pub(crate) fn number(v: f64, decimals: std::ops::RangeInclusive<usize>) -> String {
    let s = egui::emath::format_with_decimals_in_range(v, decimals);
    if vi() { s.replace('.', ",") } else { s }
}

/// A typed number: a decimal comma is accepted (and a dot, as typed on any keyboard).
pub(crate) fn parse(s: &str) -> Option<f64> {
    let s: String = s.trim().chars().filter(|c| !c.is_whitespace() && !matches!(c, '%' | '°')).collect();
    s.replace(',', ".").parse().ok().filter(|v: &f64| v.is_finite())
}

/// A drag value that shows and takes the interface's decimal separator.
pub(crate) fn drag(d: egui::DragValue<'_>) -> egui::DragValue<'_> {
    if vi() { d.custom_formatter(number).custom_parser(parse) } else { d }
}

/// A slider that shows and takes the interface's decimal separator.
pub(crate) fn slider(s: egui::Slider<'_>) -> egui::Slider<'_> {
    if vi() { s.custom_formatter(number).custom_parser(parse) } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vietnamese_starts_in_inter_until_a_font_is_picked() {
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap_or(crate::i18n::Lang::EN));
        assert_eq!(default_family(FontFamily::Helvetica), FontFamily::Inter);
        assert_eq!(default_family(FontFamily::Times), FontFamily::Times, "a chosen family stays");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        assert_eq!(default_family(FontFamily::Helvetica), FontFamily::Helvetica, "English keeps the standard default");
        assert!(families(true).contains(&FontFamily::Inter) && !families(false).contains(&FontFamily::Inter));
        for f in families(true) {
            assert!(!label(*f).is_empty());
        }
    }

    #[test]
    fn vietnamese_numbers_dates_and_units() {
        use crate::units::Unit;
        crate::i18n::set_current(crate::i18n::Lang::from_code("vi").unwrap_or(crate::i18n::Lang::EN));
        assert_eq!(number(0.5, 2..=2), "0,50");
        assert_eq!(number(100.0, 1..=1), "100,0");
        assert_eq!(parse("0,5"), Some(0.5));
        assert_eq!(parse(" 1.25 "), Some(1.25));
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("NaN"), None);
        assert_eq!(units(Unit::Inches), Unit::Centimetres);
        assert_eq!(units(Unit::Millimetres), Unit::Millimetres);
        assert_eq!(date_formats(&["a", "b", "c", "d", "e", "f"])[0], "<<dd/mm/yyyy>>");
        assert_eq!(date("2026-10-26 13:39"), "26/10/2026 13:39");
        assert_eq!(date("hôm qua"), "hôm qua");
        assert_eq!(date("2026-10-26 13:3Đ"), "2026-10-26 13:3Đ");
        crate::i18n::set_current(crate::i18n::Lang::EN);
        assert_eq!(number(0.5, 2..=2), "0.50");
        assert_eq!(units(Unit::Inches), Unit::Inches);
        assert_eq!(date_formats(&["a", "b", "c", "d", "e", "f"])[0], "a");
        assert_eq!(date("2026-10-26 13:39"), "2026-10-26 13:39");
    }
}
