//! Design tokens and egui style (plan/acrobat/02-ui-ux.md §1: neutral chrome, white panels,
//! one accent; Dark Gray keeps pages white). Every custom widget reads `Tokens::get`.
//!
//! Nexus PDF (NexusOffice): slate surfaces, the app colour Rose, and an accent colour the user
//! may choose ([`set_accent`], Preferences › Appearance, option `accent`).

use std::sync::Arc;

use egui::epaint::text::{FontTweak, HintingTarget, SmoothHinting};
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Rect, Response, Stroke, Vec2, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemeKind {
    #[default]
    Light,
    Dark,
}

/// The saved user choice, independent of the light/dark colours currently displayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    System,
    #[default]
    Light,
    Dark,
}

impl ThemePreference {
    /// The colours to draw. `system` is what the operating system says, `None` when it says
    /// nothing; `fallback` is supplied by the caller. PdfCraft uses Light for an unknown desktop.
    pub fn resolve(self, system: Option<bool>, fallback: ThemeKind) -> ThemeKind {
        match self {
            Self::Light => ThemeKind::Light,
            Self::Dark => ThemeKind::Dark,
            Self::System => match system {
                Some(true) => ThemeKind::Dark,
                Some(false) => ThemeKind::Light,
                None => fallback,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub kind: ThemeKind,
    /// Title/tab strip.
    pub titlebar: Color32,
    /// Mode bar, panels.
    pub chrome: Color32,
    pub panel: Color32,
    /// Document area behind pages.
    pub pasteboard: Color32,
    pub card: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub selected: Color32,
    /// Fills (primary buttons, badges, the active tab line) with `on_accent` text on them.
    pub accent: Color32,
    /// A hovered primary button: darker than `accent`, so `on_accent` stays readable on it.
    pub accent_hover: Color32,
    /// Accent-coloured text and links on the surfaces.
    pub accent_text: Color32,
    pub accent_soft: Color32,
    /// Text and icons drawn on an `accent` fill.
    pub on_accent: Color32,
    pub field: Color32,
    /// Information notice bars (form fields, secured, snapshot): a neutral tint and icon that
    /// never follow the accent, so an information bar doesn't read as a warning in Rose.
    pub info_bg: Color32,
    pub info_icon: Color32,
    pub badge_new: Color32,
    pub page_shadow: Color32,
    pub radius: u8,
}

impl Tokens {
    /// The Nexus PDF palettes (NexusOffice design system): slate surfaces and the app colour Rose.
    /// Fills use Rose's ink shade [`APP_INK`] (white text on it reads at 4.7:1; Rose `#F43F5E`
    /// itself only 3.7:1), accent text a shade that reads at 4.5:1 on every surface.
    pub fn for_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Light => Self {
                kind,
                titlebar: Color32::from_rgb(0xE9, 0xED, 0xF2),
                chrome: Color32::from_rgb(0xF8, 0xFA, 0xFC),
                panel: Color32::from_rgb(0xF8, 0xFA, 0xFC),
                pasteboard: Color32::from_rgb(0xF1, 0xF5, 0xF9),
                card: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                border: Color32::from_rgb(0xCB, 0xD5, 0xE1),
                divider: Color32::from_rgb(0xE2, 0xE8, 0xF0),
                text: Color32::from_rgb(0x1E, 0x29, 0x3B),
                text_muted: Color32::from_rgb(0x47, 0x55, 0x69),
                text_faint: Color32::from_rgb(0x5B, 0x6B, 0x80),
                icon: Color32::from_rgb(0x33, 0x41, 0x55),
                hover: Color32::from_rgb(0xEE, 0xF2, 0xF6),
                pressed: Color32::from_rgb(0xE2, 0xE8, 0xF0),
                selected: Color32::from_rgb(0xFF, 0xE4, 0xE6),
                accent: APP_INK,
                accent_hover: APP_INK_DEEP,
                accent_text: APP_INK_DEEP,
                accent_soft: Color32::from_rgb(0xFF, 0xF1, 0xF2),
                on_accent: Color32::WHITE,
                field: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                info_bg: Color32::from_rgb(0xEF, 0xF6, 0xFF),
                info_icon: Color32::from_rgb(0x03, 0x69, 0xA1),
                badge_new: APP_INK,
                page_shadow: Color32::from_black_alpha(34),
                radius: 6,
            },
            ThemeKind::Dark => Self {
                kind,
                titlebar: Color32::from_rgb(0x0F, 0x17, 0x2A),
                chrome: Color32::from_rgb(0x1E, 0x29, 0x3B),
                panel: Color32::from_rgb(0x1E, 0x29, 0x3B),
                pasteboard: Color32::from_rgb(0x02, 0x06, 0x17),
                card: Color32::from_rgb(0x27, 0x34, 0x49),
                border: Color32::from_rgb(0x47, 0x55, 0x69),
                divider: Color32::from_rgb(0x33, 0x41, 0x55),
                text: Color32::from_rgb(0xF1, 0xF5, 0xF9),
                text_muted: Color32::from_rgb(0xA8, 0xB4, 0xC4),
                text_faint: Color32::from_rgb(0x94, 0xA3, 0xB8),
                icon: Color32::from_rgb(0xCB, 0xD5, 0xE1),
                hover: Color32::from_rgb(0x33, 0x41, 0x55),
                pressed: Color32::from_rgb(0x47, 0x55, 0x69),
                selected: Color32::from_rgb(0x6B, 0x0F, 0x2A),
                accent: APP_INK,
                accent_hover: APP_INK_DEEP,
                accent_text: Color32::from_rgb(0xFB, 0x71, 0x85),
                accent_soft: Color32::from_rgb(0x4C, 0x05, 0x19),
                on_accent: Color32::WHITE,
                field: Color32::from_rgb(0x0F, 0x17, 0x2A),
                info_bg: Color32::from_rgb(0x1E, 0x29, 0x3B),
                info_icon: Color32::from_rgb(0x7D, 0xD3, 0xFC),
                badge_new: APP_INK,
                page_shadow: Color32::from_black_alpha(120),
                radius: 6,
            },
        }
    }

    /// The palette for `kind` in this context: [`Self::for_kind`] with the user's accent colour
    /// ([`set_accent`]), if any.
    pub fn for_context(ctx: &egui::Context, kind: ThemeKind) -> Self {
        let t = Self::for_kind(kind);
        accent(ctx).map_or(t, |c| with_accent(t, c))
    }

    pub fn get(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp::<Tokens>(egui::Id::new("pdfcraft-theme"))).unwrap_or_else(|| Self::for_kind(ThemeKind::Light))
    }

    pub fn dark(&self) -> bool {
        self.kind == ThemeKind::Dark
    }
}

/// Nexus PDF's app colour, Rose (NexusOffice design system).
pub const APP_COLOR: Color32 = Color32::from_rgb(0xF4, 0x3F, 0x5E);
/// Rose's ink shade (the design system's second colour): the fill under white text, 4.7:1.
pub const APP_INK: Color32 = Color32::from_rgb(0xE1, 0x1D, 0x48);
/// A deeper ink: accent text on light surfaces and hovered fills, 6.3:1 with white.
pub const APP_INK_DEEP: Color32 = Color32::from_rgb(0xBE, 0x12, 0x3C);

/// Colours that mean something on the page or about a state, not branding. They replace
/// upstream's values from Adobe's Spectrum palette (`#1473E6` selection blue, `#D7373F` red,
/// `#E68619` orange, `#2D9D78` green) with the NexusOffice palette's own.
/// Selection frames, handles, link and focus outlines drawn over the (white) page: 6.3:1 on it.
pub const SELECTION: Color32 = Color32::from_rgb(0x4F, 0x46, 0xE5);
/// Information (a certified document): 5.2:1 on white.
pub const INFO: Color32 = Color32::from_rgb(0x25, 0x63, 0xEB);
/// Errors and failures (invalid signatures, failed checks): 4.8:1 on white.
pub const ERROR: Color32 = Color32::from_rgb(0xDC, 0x26, 0x26);
/// Warnings (a signature with problems, a file that needs attention).
pub const WARNING: Color32 = Color32::from_rgb(0xD9, 0x77, 0x06);
/// Success (valid signatures).
pub const SUCCESS: Color32 = Color32::from_rgb(0x05, 0x96, 0x69);

/// Accent presets offered in Preferences › Appearance (English and Vietnamese names); any other
/// colour comes from the picker. Coral is the app colour's own hue.
pub const ACCENT_PRESETS: [(&str, &str, Color32); 6] = [
    ("Indigo", "Chàm", Color32::from_rgb(0x63, 0x66, 0xF1)),
    ("Violet", "Tím", Color32::from_rgb(0x8B, 0x5C, 0xF6)),
    ("Mint", "Bạc hà", Color32::from_rgb(0x10, 0xB9, 0x81)),
    ("Amber", "Hổ phách", Color32::from_rgb(0xF5, 0x9E, 0x0B)),
    ("Coral", "San hô", APP_COLOR),
    ("Sky", "Da trời", Color32::from_rgb(0x0E, 0xA5, 0xE9)),
];

/// Where the user's accent colour lives: the egui context's memory, so every widget reads it
/// through [`Tokens::get`] and each window (and each test harness) has its own.
fn accent_id() -> egui::Id {
    egui::Id::new("nexus-user-accent")
}

/// The user's accent colour in this context, if any.
pub fn accent(ctx: &egui::Context) -> Option<Color32> {
    ctx.data(|d| d.get_temp::<Color32>(accent_id()))
}

/// Sets (or with `None` clears) the user's accent colour and re-applies the current theme with
/// it. Returns whether it changed; setting the same colour again does nothing.
pub fn set_accent(ctx: &egui::Context, c: Option<Color32>) -> bool {
    if accent(ctx) == c {
        return false;
    }
    ctx.data_mut(|d| match c {
        Some(c) => {
            d.insert_temp(accent_id(), c);
        }
        None => d.remove::<Color32>(accent_id()),
    });
    apply(ctx, Tokens::get(ctx).kind);
    ctx.request_repaint();
    true
}

/// `#RRGGBB` or `RRGGBB` (surrounding spaces allowed) → colour; anything else `None`.
pub fn parse_hex(s: &str) -> Option<Color32> {
    let h = s.trim();
    let h = h.strip_prefix('#').unwrap_or(h);
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    // Six ASCII hex digits: every two-byte slice is on a char boundary.
    let byte = |i: usize| h.get(i..i + 2).and_then(|x| u8::from_str_radix(x, 16).ok());
    Some(Color32::from_rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// `#RRGGBB` for a colour (alpha ignored).
pub fn to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn luminance(c: Color32) -> f32 {
    let ch = |v: u8| {
        let s = f32::from(v) / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
}

/// WCAG 2 contrast ratio of two opaque colours (1 to 21).
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `c` moved towards black (or white with `lighten`) in 2.5% steps until it reads against every
/// colour in `against` at `ratio`. Black or white ends the walk, so it always stops.
fn shade_for(c: Color32, against: &[Color32], ratio: f32, lighten: bool) -> Color32 {
    let target = if lighten { 255.0 } else { 0.0 };
    let mix = |v: u8, k: f32| {
        let v = f32::from(v);
        (v + (target - v) * k).round().clamp(0.0, 255.0) as u8
    };
    for step in 0..=40u8 {
        let k = f32::from(step) * 0.025;
        let out = Color32::from_rgb(mix(c.r(), k), mix(c.g(), k), mix(c.b(), k));
        if against.iter().all(|b| contrast_ratio(out, *b) >= ratio) {
            return out;
        }
    }
    if lighten { Color32::WHITE } else { Color32::BLACK }
}

/// `c` mixed with white (`k` = how much of `c` stays).
fn tint(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (255.0 - (255.0 - f32::from(v)) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// `c` mixed with black (`k` = how much of `c` stays).
fn shade(c: Color32, k: f32) -> Color32 {
    let m = |v: u8| (f32::from(v) * k).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// The palette with the user's accent colour `c` in place of the app colour. Every colour drawn
/// as text or under text is shaded until it reads at WCAG 4.5:1: `accent` (and the darker
/// `accent_hover`, `badge_new`) under the white `on_accent` text, `accent_text` on every surface
/// and on the soft and selected tints, the text on those tints.
pub fn with_accent(mut t: Tokens, c: Color32) -> Tokens {
    let c = Color32::from_rgb(c.r(), c.g(), c.b());
    t.on_accent = Color32::WHITE;
    t.accent = shade_for(c, &[t.on_accent], 4.5, false);
    t.accent_hover = shade(t.accent, 0.85);
    t.badge_new = t.accent;
    if t.dark() {
        t.accent_soft = shade(c, 0.30);
        t.selected = shade(c, 0.40);
    } else {
        t.accent_soft = tint(c, 0.10);
        t.selected = tint(c, 0.16);
    }
    let surfaces = [t.titlebar, t.chrome, t.panel, t.card, t.pasteboard, t.field, t.accent_soft, t.selected];
    t.accent_text = shade_for(c, &surfaces, 4.5, t.dark());
    t
}

/// Hover and press amounts for chrome controls. The wash eases in; holding the pointer
/// darkens it and drops the content one pixel, so a click reads as a key settling.
#[derive(Clone, Copy, Debug)]
pub struct Press {
    pub hover: f32,
    pub down: f32,
}

impl Press {
    pub fn track(ui: &egui::Ui, resp: &Response) -> Self {
        let hot = resp.hovered() || resp.is_pointer_button_down_on();
        let ease = egui::emath::easing::quadratic_out;
        // With animation effects off the wash and the press are simply there.
        let (hover, down) = if crate::motion::animated(ui.ctx()) { (0.12, 0.06) } else { (0.0, 0.0) };
        Self {
            hover: ui.ctx().animate_bool_with_time_and_easing(resp.id.with("chrome-hover"), hot, hover, ease),
            down: ui.ctx().animate_bool_with_time_and_easing(resp.id.with("chrome-down"), resp.is_pointer_button_down_on(), down, ease),
        }
    }

    pub fn fill(self, rest: Color32, hover: Color32, pressed: Color32) -> Color32 {
        rest.lerp_to_gamma(hover, self.hover).lerp_to_gamma(pressed, self.down)
    }

    /// One pixel down at a full press.
    pub fn offset(self) -> Vec2 {
        Vec2::new(0.0, self.down)
    }

    /// The colour [`Self::wash`] paints.
    pub fn wash_color(self, t: &Tokens, selected: bool) -> Color32 {
        if selected {
            self.fill(t.accent_soft, t.accent_soft.lerp_to_gamma(t.accent, 0.16), t.selected)
        } else {
            self.fill(Color32::TRANSPARENT, t.hover, t.pressed)
        }
    }

    /// Soft wash behind a control. `selected` keeps the accent fill.
    pub fn wash(self, ui: &egui::Ui, rect: Rect, radius: impl Into<CornerRadius>, selected: bool) {
        let fill = self.wash_color(&Tokens::get(ui.ctx()), selected);
        if fill.a() > 0 {
            ui.painter().rect_filled(rect, radius, fill);
        }
    }
}

/// Pointing hand, the cursor for a control that clicks.
pub fn hand(resp: Response) -> Response {
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn install_fonts(ctx: &egui::Context) {
    install_fonts_for(ctx, CjkPreference::Default);
}

/// The interface language's claim on the CJK fallback faces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CjkPreference {
    /// Japanese faces first (every non-Chinese language).
    #[default]
    Default,
    SimplifiedChinese,
    TraditionalChinese,
}

impl CjkPreference {
    /// The preference for a UI language code (`zh-hans`, `zh-hant`, anything else).
    pub fn for_language(code: &str) -> Self {
        match code {
            "zh-hans" => Self::SimplifiedChinese,
            "zh-hant" => Self::TraditionalChinese,
            _ => Self::Default,
        }
    }
}

/// Install the interface fonts with the CJK fallback order for the UI language (Chinese
/// first for Chinese, Japanese first otherwise). Call it when the language changes; the new
/// faces take effect next frame.
pub fn install_fonts_for(ctx: &egui::Context, cjk: CjkPreference) {
    ctx.set_fonts(installed_font_definitions(cjk));
}

/// NexusOffice: [`install_fonts_for`] for an app embedded in a host window with other editors
/// (their tabs share one egui context). The window's other faces and families (another editor's,
/// the host's) stay; this app's faces are added and its families set, so switching tabs never
/// takes a family away from the editor that draws with it.
pub fn install_fonts_beside(ctx: &egui::Context, cjk: CjkPreference) {
    ctx.set_fonts(fonts_beside(ctx, cjk));
}

/// What [`install_fonts_beside`] installs: the window's fonts with this app's added.
pub fn fonts_beside(ctx: &egui::Context, cjk: CjkPreference) -> FontDefinitions {
    let ours = installed_font_definitions(cjk);
    let mut merged = ctx.fonts(|f| f.definitions().clone());
    merged.font_data.extend(ours.font_data);
    for (family, chain) in ours.families {
        merged.families.insert(family, chain);
    }
    merged
}

/// NexusOffice: whether the window's fonts have this app's named families ([`medium`],
/// [`semibold`]); another editor in a host window may have replaced them.
pub fn fonts_installed(ctx: &egui::Context) -> bool {
    ctx.fonts(|f| {
        let families = &f.definitions().families;
        ["medium", "semibold"].iter().all(|name| families.contains_key(&FontFamily::Name((*name).into())))
    })
}

/// The name of the installed face [`installed_font_definitions`] may add after the embedded ones.
pub const SYSTEM_FALLBACK: &str = "system-fallback";
/// The name of the installed Devanagari (Hindi) face [`installed_font_definitions`] may add.
pub const SYSTEM_DEVANAGARI_FALLBACK: &str = "system-devanagari-fallback";
/// The name of the installed Telugu face [`installed_font_definitions`] may add.
pub const SYSTEM_TELUGU_FALLBACK: &str = "system-telugu-fallback";
/// The name of the installed Sinhala face [`installed_font_definitions`] may add.
pub const SYSTEM_SINHALA_FALLBACK: &str = "system-sinhala-fallback";
/// The name of the installed Tamil face [`installed_font_definitions`] may add.
pub const SYSTEM_TAMIL_FALLBACK: &str = "system-tamil-fallback";

/// What [`install_fonts_for`] installs: [`font_definitions_for`], then, on desktop, one face
/// already installed on this machine. It only draws characters no embedded face has.
///
/// - In a Chinese interface built without a `Hans` craft-fonts face (every release so far,
///   #826), it is a Chinese system face (Microsoft YaHei, PingFang, …), placed before the
///   Japanese craft-fonts faces: they lack hundreds of the catalog's characters (欢, 导, 签, …),
///   and one line drawn from two faces with different vertical metrics would not sit on one
///   baseline.
/// - Otherwise it is the last fallback of every family (an Arabic file name in a build without
///   craft-fonts).
///
/// Installed Devanagari, Telugu, Sinhala and Tamil faces, when there are, end every family just
/// before that one, so Hindi, Telugu (#1531), Sinhala and Tamil (#1688) text, such as a file
/// name, draws in a build without such faces.
///
/// `PDFCRAFT_SYSTEM_FONTS=0` leaves them out.
pub fn installed_font_definitions(cjk: CjkPreference) -> FontDefinitions {
    #[cfg_attr(target_arch = "wasm32", expect(unused_mut))]
    let mut fonts = font_definitions_for(cjk == CjkPreference::SimplifiedChinese);
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::system_fonts::{Need, fallback};
        // Before the general installed face, which stays the last fallback outside Chinese.
        for (name, data) in [
            (SYSTEM_DEVANAGARI_FALLBACK, crate::system_fonts::devanagari_fallback()),
            (SYSTEM_TELUGU_FALLBACK, crate::system_fonts::telugu_fallback()),
            (SYSTEM_SINHALA_FALLBACK, crate::system_fonts::sinhala_fallback()),
            (SYSTEM_TAMIL_FALLBACK, crate::system_fonts::tamil_fallback()),
        ] {
            let Some(data) = data else { continue };
            fonts.font_data.insert(name.to_owned(), data);
            for stack in fonts.families.values_mut() {
                stack.push(name.to_owned());
            }
        }
        let need = match cjk {
            _ if !pdfcraft_fonts::ui_chinese_fonts().is_empty() => Need::Other,
            CjkPreference::SimplifiedChinese => Need::Hans,
            CjkPreference::TraditionalChinese => Need::Hant,
            CjkPreference::Default => Need::Other,
        };
        if let Some(data) = fallback(need) {
            fonts.font_data.insert(SYSTEM_FALLBACK.to_owned(), data);
            let japanese: Vec<String> = pdfcraft_fonts::ui_japanese_fonts().iter().map(|f| f.name()).collect();
            for stack in fonts.families.values_mut() {
                let at = match need {
                    Need::Other => None,
                    Need::Hans | Need::Hant => stack.iter().position(|n| japanese.contains(n)),
                };
                stack.insert(at.unwrap_or(stack.len()), SYSTEM_FALLBACK.to_owned());
            }
        }
    }
    fonts
}

/// The interface fonts: Inter (and JetBrains Mono for code) first, then egui's defaults, then
/// the CJK, Arabic, Telugu, Hebrew and Thai faces of the optional craft-fonts build input as the
/// last fallback in every family. Without craft-fonts there is no Japanese, Chinese, Arabic,
/// Telugu, Hebrew or Thai face here.
pub fn font_definitions() -> FontDefinitions {
    font_definitions_for(false)
}

/// [`font_definitions`] with the CJK fallback order for the UI language. Simplified Chinese
/// must come first in Chinese mode: otherwise shared characters render in the Japanese face
/// while Simplified-only characters (e.g. U+6B22 欢) fall through to the Chinese face, and
/// the mixed vertical metrics sink them below the line.
pub fn font_definitions_for(prefer_hans: bool) -> FontDefinitions {
    font_definitions_for_platform(prefer_hans, current_font_policy_platform())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FontPolicyPlatform {
    Windows,
    Other,
}

fn current_font_policy_platform() -> FontPolicyPlatform {
    if cfg!(target_os = "windows") { FontPolicyPlatform::Windows } else { FontPolicyPlatform::Other }
}

fn primary_font_tweak(platform: FontPolicyPlatform, name: &str) -> FontTweak {
    let mut tweak = FontTweak::default();
    if platform == FontPolicyPlatform::Windows && name == "JetBrainsMono" {
        tweak.hinting_target = HintingTarget::Smooth(SmoothHinting { symmetric_rendering: false, ..SmoothHinting::default() });
    }
    tweak
}

/// A face's vertical metrics in em, as egui reads them: skrifa's ascent, descent (negative) and
/// line gap at the default location. `None` when the face doesn't parse.
fn vertical_metrics(bytes: &[u8], index: u32) -> Option<[f32; 3]> {
    use skrifa::MetadataProvider as _;
    let font = skrifa::FontRef::from_index(bytes, index).ok()?;
    let m = font.metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
    let em = f32::from(m.units_per_em);
    let metrics = [m.ascent / em, m.descent / em, m.leading / em];
    (em > 0.0 && metrics.iter().all(|v| v.is_finite())).then_some(metrics)
}

/// How far below the primary face's baseline egui draws a fallback face's, in em of the font size
/// (negative: above). egui centres the fallback's line box (ascent − descent + gap) on the
/// primary's rather than lining up their baselines, so the baseline moves by half the difference
/// of ascent + descent, less half the difference of the line gaps (epaint `text_layout`).
fn baseline_drop(primary: [f32; 3], fallback: [f32; 3]) -> f32 {
    let [pa, pd, pg] = primary;
    let [a, d, g] = fallback;
    ((a + d) - (pa + pd) + pg - g) / 2.0
}

/// `data` with its glyphs moved onto Inter's baseline (#1676). Without this, Chinese next to Latin
/// in one label sits higher or lower than the Latin by a few percent of the font size, depending on
/// the installed Chinese face (PingFang, Microsoft JhengHei, SimSun…). Inter is the first face of
/// every proportional family; the monospace family's JetBrains Mono differs from it by 0.004 em,
/// well under a pixel. A face that doesn't parse is left as it is (egui will skip it anyway).
pub(crate) fn on_interface_baseline(mut data: FontData) -> FontData {
    let primary = vertical_metrics(pdfcraft_fonts::INTER_REGULAR, 0);
    if let (Some(primary), Some(fallback)) = (primary, vertical_metrics(&data.font, data.index)) {
        data.tweak.y_offset_factor = -baseline_drop(primary, fallback);
    }
    data
}

fn font_definitions_for_platform(prefer_hans: bool, platform: FontPolicyPlatform) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let add = |fonts: &mut FontDefinitions, name: &str, bytes: &'static [u8]| {
        let data = FontData::from_static(bytes);
        let data = if ["Inter", "Inter-Medium", "Inter-SemiBold", "JetBrainsMono"].contains(&name) {
            data.tweak(primary_font_tweak(platform, name))
        } else {
            on_interface_baseline(data)
        };
        fonts.font_data.insert(name.to_owned(), Arc::new(data));
    };
    // Shared with typed signatures' fallback glyphs, so the font is embedded once.
    add(&mut fonts, "Inter", pdfcraft_fonts::INTER_REGULAR);
    add(&mut fonts, "Inter-Medium", include_bytes!("../../../assets/fonts/Inter-Medium.ttf"));
    add(&mut fonts, "Inter-SemiBold", include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf"));
    add(&mut fonts, "JetBrainsMono", include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf"));
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Inter".to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "JetBrainsMono".to_owned());
    // Keep missing Latin/Cyrillic letters in the app's own faces (for example Bulgarian ѝ,
    // absent from JetBrains Mono), before egui's defaults or optional system/CJK fallbacks.
    fonts.families.entry(FontFamily::Monospace).or_default().insert(1, "Inter".to_owned());
    // The same static bytes pdfcraft-fonts uses for Japanese/Chinese text in PDFs: one copy, not two.
    for face in pdfcraft_fonts::ui_cjk_fonts(prefer_hans) {
        let name = face.name();
        // Japanese and Chinese faces have distinct family names, so no collision here.
        if !fonts.font_data.contains_key(&name) {
            add(&mut fonts, &name, face.bytes);
        }
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push(name.clone());
        }
    }
    // Arabic-script faces (file names, document titles), Telugu faces (the Telugu interface,
    // file names), Hebrew faces (file names, text being edited) and Thai faces after the CJK ones; the
    // ranges don't overlap.
    for face in pdfcraft_fonts::ui_arabic_fonts()
        .into_iter()
        .chain(pdfcraft_fonts::ui_telugu_fonts())
        .chain(pdfcraft_fonts::ui_hebrew_fonts())
        .chain(pdfcraft_fonts::ui_thai_fonts())
    {
        let name = face.name();
        if !fonts.font_data.contains_key(&name) {
            add(&mut fonts, &name, face.bytes);
        }
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            let stack = fonts.families.entry(family).or_default();
            if !stack.contains(&name) {
                stack.push(name.clone());
            }
        }
    }
    let fallback: Vec<String> = fonts.families[&FontFamily::Proportional].clone();
    for (fam, primary) in [("medium", "Inter-Medium"), ("semibold", "Inter-SemiBold")] {
        let mut stack = vec![primary.to_owned()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(fam.into()), stack);
    }
    fonts
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

pub fn apply(ctx: &egui::Context, kind: ThemeKind) {
    // Keep the native window inherited. A concrete egui theme pins macOS's
    // NSWindow appearance and prevents winit from delivering ThemeChanged.
    // Our saved choice owns the palette; both egui branches receive it below.
    ctx.set_theme(egui::ThemePreference::System);
    let t = Tokens::for_context(ctx, kind);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("pdfcraft-theme"), t));
    let mut v = if t.dark() { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.panel;
    v.window_fill = t.card;
    v.window_stroke = Stroke::new(1.0, t.border);
    v.extreme_bg_color = t.field;
    v.faint_bg_color = t.hover;
    v.selection.bg_fill = t.accent_soft;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.hyperlink_color = t.accent_text;
    v.override_text_color = Some(t.text);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.window_shadow = egui::Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(if t.dark() { 110 } else { 38 }) };
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(if t.dark() { 90 } else { 30 }) };
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = CornerRadius::same(t.radius);
        w.fg_stroke.color = t.text;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.divider);
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_fill = t.field;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.weak_bg_fill = t.hover;
    v.widgets.hovered.bg_fill = t.hover;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.active.weak_bg_fill = t.pressed;
    v.widgets.active.bg_fill = t.pressed;
    // Stock buttons (menus, dialogs) use the same pointing hand as the custom chrome.
    v.interact_cursor = Some(egui::CursorIcon::PointingHand);
    v.widgets.open.weak_bg_fill = t.hover;
    // Crisper text at 100–150 % scaling (#76): glyphs sit on whole pixels instead of being
    // rendered at quarter-pixel offsets, which egui notes blurs them. In the light theme, a mild
    // gamma darkens the antialiased edges of dark text (egui's default is linear, which reads thin
    // and grey next to the system's text); the dark theme keeps egui's own curve. At 200 % both
    // make little difference.
    v.text_options.subpixel_binning = false;
    if !t.dark() {
        v.text_options.color_transfer_function = egui::epaint::FontColorTransferFunction::Gamma(0.75);
    }
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.scroll.bar_width = 8.0;
        s.spacing.scroll.floating = true;
        s.spacing.scroll.floating_width = 3.0;
        s.spacing.scroll.foreground_color = true;
        s.text_styles.insert(egui::TextStyle::Body, regular(13.0));
        s.text_styles.insert(egui::TextStyle::Button, regular(13.0));
        s.text_styles.insert(egui::TextStyle::Small, regular(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, semibold(17.0));
        s.interaction.tooltip_delay = 0.35;
    });
    let style = ctx.global_style();
    ctx.set_style_of(egui::Theme::Light, style.clone());
    ctx.set_style_of(egui::Theme::Dark, style);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::epaint::text::{Fonts, TextOptions};

    const HINTING_CORPUS: [&str; 4] = ["Il1MWmw", "Hamburgefonts", "0123456789 .,;:!?()[]{}+-=/\\", "Café Ångström déjà-vu Ωμέγα Ελληνικά Журнал"];
    const HINTING_SIZES: [f32; 6] = [10.0, 11.0, 12.0, 13.0, 14.0, 16.0];
    const HINTING_PPPS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

    fn primary_families() -> [(FontFamily, &'static str); 4] {
        [
            (FontFamily::Proportional, "Inter"),
            (FontFamily::Name("medium".into()), "Inter-Medium"),
            (FontFamily::Name("semibold".into()), "Inter-SemiBold"),
            (FontFamily::Monospace, "JetBrainsMono"),
        ]
    }

    fn raster_edge_and_ink(fonts: &mut Fonts, id: FontId, text: &str, ppp: f32) -> (u64, u64) {
        let galley = {
            let mut view = fonts.with_pixels_per_point(ppp);
            view.layout_no_wrap(text.to_owned(), id, Color32::BLACK).as_ref().clone()
        };
        let image = fonts.image();
        let mut edge = 0u64;
        let mut ink = 0u64;
        for glyph in galley.rows.iter().flat_map(|row| &row.glyphs) {
            let [x0, y0] = glyph.uv_rect.min.map(usize::from);
            let [x1, y1] = glyph.uv_rect.max.map(usize::from);
            for y in y0..y1 {
                for x in x0..x1 {
                    let alpha = image.pixels[y * image.size[0] + x].a();
                    ink += u64::from(alpha);
                    if x > x0 {
                        let left = image.pixels[y * image.size[0] + x - 1].a();
                        edge += u64::from(alpha.abs_diff(left));
                    }
                }
            }
        }
        (edge, ink)
    }

    fn mono_raster_measure(fonts: &mut Fonts, ppp: f32) -> (u64, u64) {
        let mut edge = 0;
        let mut ink = 0;
        for size in HINTING_SIZES {
            for text in HINTING_CORPUS {
                let (glyph_edge, glyph_ink) = raster_edge_and_ink(fonts, FontId::monospace(size), text, ppp);
                edge += glyph_edge;
                ink += glyph_ink;
            }
        }
        (edge, ink)
    }

    fn layout_signature(galley: &egui::Galley) -> Vec<u32> {
        let mut signature = vec![
            galley.rect.min.x.to_bits(),
            galley.rect.min.y.to_bits(),
            galley.rect.max.x.to_bits(),
            galley.rect.max.y.to_bits(),
            u32::from(galley.elided),
            galley.rows.len() as u32,
        ];
        for row in &galley.rows {
            signature.extend([
                u32::from(row.ends_with_newline),
                row.pos.x.to_bits(),
                row.pos.y.to_bits(),
                row.size.x.to_bits(),
                row.size.y.to_bits(),
                row.glyphs.len() as u32,
            ]);
            for glyph in &row.glyphs {
                signature.extend([
                    glyph.chr as u32,
                    glyph.pos.x.to_bits(),
                    glyph.pos.y.to_bits(),
                    glyph.advance_width.to_bits(),
                    glyph.line_height.to_bits(),
                    glyph.font_ascent.to_bits(),
                    glyph.font_height.to_bits(),
                    glyph.font_face_ascent.to_bits(),
                    glyph.font_face_height.to_bits(),
                ]);
            }
        }
        signature
    }

    fn galley(fonts: &mut Fonts, family: FontFamily, size: f32, ppp: f32, text: &str) -> egui::Galley {
        let mut view = fonts.with_pixels_per_point(ppp);
        view.layout_no_wrap(text.to_owned(), FontId::new(size, family), Color32::BLACK).as_ref().clone()
    }

    fn hinting_test_fonts(platform: FontPolicyPlatform) -> Fonts {
        Fonts::new(
            TextOptions {
                subpixel_binning: false,
                color_transfer_function: egui::epaint::FontColorTransferFunction::Gamma(0.75),
                ..TextOptions::default()
            },
            font_definitions_for_platform(false, platform),
        )
    }

    /// The baseline of each glyph of `text` laid out by egui at `size` points, in points from the
    /// top of the row: its position plus its face's `y_offset_factor`, as epaint draws it.
    fn baselines(fallback: FontData, size: f32, text: &str) -> Vec<f32> {
        let factor = fallback.tweak.y_offset_factor;
        let mut defs = FontDefinitions::empty();
        defs.font_data.insert("Inter".into(), Arc::new(FontData::from_static(pdfcraft_fonts::INTER_REGULAR)));
        defs.font_data.insert("fallback".into(), Arc::new(fallback));
        defs.families.insert(FontFamily::Proportional, vec!["Inter".into(), "fallback".into()]);
        let mut fonts = Fonts::new(TextOptions::default(), defs);
        let g = galley(&mut fonts, FontFamily::Proportional, size, 1.0, text);
        let inter_has = |c: char| c.is_ascii();
        g.rows[0].glyphs.iter().map(|gl| gl.pos.y + if inter_has(gl.chr) { 0.0 } else { size * factor }).collect()
    }

    #[test]
    fn fallback_faces_share_the_interface_baseline() {
        // Dancing Script's metrics differ from Inter's the way a Chinese face's do (by 0.044 em),
        // and it has letters Inter doesn't (Ǆ), which egui then takes from it.
        let dancing: &'static [u8] = include_bytes!("../../../assets/fonts/DancingScript.ttf");
        let size = 100.0;
        // As egui lays it out by itself, Ǆ sits ~4 pt off H's baseline: the bug (#1676).
        let raw = baselines(FontData::from_static(dancing), size, "HǄ");
        assert!((raw[1] - raw[0]).abs() > 3.0, "the test fonts must differ: {raw:?}");
        let drop = baseline_drop(vertical_metrics(pdfcraft_fonts::INTER_REGULAR, 0).unwrap(), vertical_metrics(dancing, 0).unwrap());
        assert!((raw[1] - raw[0] - size * drop).abs() <= 1.0, "baseline_drop predicts egui: {raw:?} vs {drop}");
        // On the interface baseline, to the pixel.
        let aligned = baselines(on_interface_baseline(FontData::from_static(dancing)), size, "HǄ");
        assert!((aligned[1] - aligned[0]).abs() <= 1.0, "{aligned:?}");
        // A face that doesn't parse is left alone rather than guessed at.
        assert_eq!(on_interface_baseline(FontData::from_static(b"not a font")).tweak, FontTweak::default());
        assert_eq!(vertical_metrics(b"", 0), None);
    }

    #[test]
    fn windows_hinting_changes_only_the_jetbrains_symmetric_rendering_field() {
        let baseline = font_definitions_for_platform(false, FontPolicyPlatform::Other);
        let windows = font_definitions_for_platform(false, FontPolicyPlatform::Windows);

        assert_eq!(baseline.families, windows.families);
        assert_eq!(baseline.font_data.keys().collect::<Vec<_>>(), windows.font_data.keys().collect::<Vec<_>>());
        for (family, name) in primary_families() {
            let baseline_data = &baseline.font_data[name];
            let windows_data = &windows.font_data[name];
            assert_eq!(baseline_data.tweak, FontTweak::default(), "baseline {name}");
            if name == "JetBrainsMono" {
                assert_eq!(
                    windows_data.tweak,
                    FontTweak {
                        hinting_target: HintingTarget::Smooth(SmoothHinting { symmetric_rendering: false, ..SmoothHinting::default() }),
                        ..FontTweak::default()
                    },
                    "Windows {name}"
                );
            } else {
                assert_eq!(windows_data.tweak, baseline_data.tweak, "Windows {name}");
            }
            assert_eq!(primary_font_tweak(FontPolicyPlatform::Other, name), baseline_data.tweak, "non-Windows policy for {family:?}");
        }
        for (name, data) in &baseline.font_data {
            if !["Inter", "Inter-Medium", "Inter-SemiBold", "JetBrainsMono"].contains(&name.as_str()) {
                assert_eq!(windows.font_data[name].tweak, data.tweak, "Windows fallback {name}");
            }
        }
        assert_eq!(
            font_definitions_for(false).font_data["JetBrainsMono"].tweak,
            font_definitions_for_platform(false, current_font_policy_platform()).font_data["JetBrainsMono"].tweak
        );
    }

    #[test]
    #[ignore = "measurement probe: asserts exact egui rasterisation deltas, which change with egui upgrades; run with --ignored when retuning the hinting"]
    fn windows_hinting_improves_low_dpi_mono_raster_without_changing_layout_or_coverage() {
        let mut baseline = hinting_test_fonts(FontPolicyPlatform::Other);
        let mut windows = hinting_test_fonts(FontPolicyPlatform::Windows);

        for ppp in HINTING_PPPS {
            let (baseline_edge, baseline_ink) = mono_raster_measure(&mut baseline, ppp);
            let (windows_edge, windows_ink) = mono_raster_measure(&mut windows, ppp);
            // Compare horizontal alpha-edge energy per ink alpha; this is a deterministic
            // vertical-stem sharpness proxy, not an absolute typography quality score.
            let baseline_sharpness = baseline_edge as f64 / baseline_ink as f64;
            let windows_sharpness = windows_edge as f64 / windows_ink as f64;
            let sharpness_change = windows_sharpness / baseline_sharpness - 1.0;
            let weight_change = windows_ink as f64 / baseline_ink as f64 - 1.0;
            if ppp < 2.0 {
                assert!(sharpness_change > 0.002, "{ppp} ppp sharpness change: {sharpness_change:.2}%", sharpness_change = sharpness_change * 100.0);
            } else {
                assert!(sharpness_change >= -0.01, "{ppp} ppp sharpness change: {sharpness_change:.2}%", sharpness_change = sharpness_change * 100.0);
            }
            assert!(weight_change.abs() < 0.05, "{ppp} ppp ink change: {weight_change:.2}%", weight_change = weight_change * 100.0);
        }

        for ppp in HINTING_PPPS {
            for size in HINTING_SIZES {
                for (family, _) in primary_families() {
                    for text in HINTING_CORPUS {
                        assert_eq!(
                            baseline.has_glyphs(&FontId::new(size, family.clone()), text),
                            windows.has_glyphs(&FontId::new(size, family.clone()), text),
                            "{family:?} {text:?}"
                        );
                        assert_eq!(
                            layout_signature(&galley(&mut baseline, family.clone(), size, ppp, text)),
                            layout_signature(&galley(&mut windows, family.clone(), size, ppp, text)),
                            "{family:?}, {size} pt, {ppp} ppp, {text:?}"
                        );
                    }
                }
            }
        }
    }

    /// "Use system setting" draws what the operating system says, dark or light.
    #[test]
    fn following_the_system_draws_what_the_system_says() {
        assert_eq!(ThemePreference::System.resolve(Some(true), ThemeKind::Light), ThemeKind::Dark);
        assert_eq!(ThemePreference::System.resolve(Some(false), ThemeKind::Dark), ThemeKind::Light);
    }

    /// An operating system that says nothing leaves the last choice standing, and an explicit
    /// Light or Dark is never overridden by what the system says.
    #[test]
    fn an_answerless_system_leaves_the_choice_and_an_explicit_one_stands() {
        assert_eq!(ThemePreference::System.resolve(None, ThemeKind::Light), ThemeKind::Light);
        assert_eq!(ThemePreference::System.resolve(None, ThemeKind::Dark), ThemeKind::Dark);
        assert_eq!(ThemePreference::Light.resolve(Some(true), ThemeKind::Dark), ThemeKind::Light);
        assert_eq!(ThemePreference::Dark.resolve(Some(false), ThemeKind::Light), ThemeKind::Dark);
    }

    /// WCAG 2 contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let lin = |v: u8| {
                let s = v as f32 / 255.0;
                if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
        };
        let (x, y) = (lum(a) + 0.05, lum(b) + 0.05);
        x.max(y) / x.min(y)
    }

    #[test]
    fn press_darkens_then_sinks() {
        let rest = Color32::from_rgb(0xFF, 0xFF, 0xFF);
        let hover = Color32::from_rgb(0xF0, 0xF0, 0xF3);
        let pressed = Color32::from_rgb(0xE4, 0xE4, 0xE9);
        let idle = Press { hover: 0.0, down: 0.0 };
        assert_eq!(idle.fill(rest, hover, pressed), rest);
        assert_eq!(idle.offset(), Vec2::ZERO);
        let held = Press { hover: 1.0, down: 1.0 };
        assert_eq!(held.fill(rest, hover, pressed), pressed);
        assert_eq!(held.offset(), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn text_is_readable_on_every_surface() {
        // #76: the faintest text (hints, zoom level, captions) was 3.3:1 in the light theme.
        for kind in [ThemeKind::Light, ThemeKind::Dark] {
            let t = Tokens::for_kind(kind);
            for (name, fg) in [("text", t.text), ("text_muted", t.text_muted), ("text_faint", t.text_faint)] {
                for bg in [t.chrome, t.panel, t.card, t.pasteboard] {
                    let r = contrast(fg, bg);
                    assert!(r >= 4.5, "{kind:?} {name} on {bg:?}: {r:.2}:1, WCAG AA needs 4.5:1");
                }
            }
        }
    }
}

#[cfg(test)]
mod nexus_palette_tests {
    use super::*;

    /// Brand colours of other vendors' office and PDF apps: Acrobat/Adobe reds and Spectrum's
    /// blue, red, orange and green, Word/Excel/PowerPoint blue, green and orange, upstream's blue.
    const VENDOR: [(u8, u8, u8); 15] = [
        (0xFA, 0x0F, 0x00),
        (0xEB, 0x10, 0x00),
        (0xF4, 0x0F, 0x02),
        (0xB3, 0x0B, 0x00),
        (0xD7, 0x37, 0x3F),
        (0x14, 0x73, 0xE6),
        (0xE6, 0x86, 0x19),
        (0x2D, 0x9D, 0x78),
        (0x01, 0x65, 0xDD),
        (0x2B, 0x57, 0x9A),
        (0x21, 0x73, 0x46),
        (0xB7, 0x47, 0x2A),
        (0x18, 0x5A, 0xBD),
        (0x10, 0x7C, 0x41),
        (0xC4, 0x3E, 0x1C),
    ];

    fn colours(t: &Tokens) -> Vec<(&'static str, Color32)> {
        vec![
            ("titlebar", t.titlebar),
            ("chrome", t.chrome),
            ("panel", t.panel),
            ("pasteboard", t.pasteboard),
            ("card", t.card),
            ("border", t.border),
            ("divider", t.divider),
            ("text", t.text),
            ("text_muted", t.text_muted),
            ("text_faint", t.text_faint),
            ("icon", t.icon),
            ("hover", t.hover),
            ("pressed", t.pressed),
            ("selected", t.selected),
            ("accent", t.accent),
            ("accent_hover", t.accent_hover),
            ("accent_text", t.accent_text),
            ("accent_soft", t.accent_soft),
            ("on_accent", t.on_accent),
            ("field", t.field),
            ("info_bg", t.info_bg),
            ("info_icon", t.info_icon),
            ("badge_new", t.badge_new),
        ]
    }

    /// No palette colour, and no semantic colour, is another vendor's brand colour (or one
    /// within a few steps of it).
    #[test]
    fn the_palettes_hold_no_vendor_brand_colour() {
        let near = |c: Color32, (r, g, b): (u8, u8, u8)| c.r().abs_diff(r) <= 6 && c.g().abs_diff(g) <= 6 && c.b().abs_diff(b) <= 6;
        let semantic = [("SELECTION", SELECTION), ("INFO", INFO), ("ERROR", ERROR), ("WARNING", WARNING), ("SUCCESS", SUCCESS)];
        for kind in [ThemeKind::Light, ThemeKind::Dark] {
            let t = Tokens::for_kind(kind);
            for (name, c) in colours(&t).into_iter().chain(semantic).chain([("APP_COLOR", APP_COLOR), ("APP_INK", APP_INK)]) {
                for v in VENDOR {
                    assert!(!near(c, v), "{kind:?} {name} {} is a vendor brand colour", to_hex(c));
                }
            }
        }
        assert_eq!(to_hex(APP_COLOR), "#F43F5E");
        assert_eq!(to_hex(APP_INK), "#E11D48");
        assert_eq!(Tokens::for_kind(ThemeKind::Light).accent, APP_INK);
        assert_eq!(Tokens::for_kind(ThemeKind::Dark).accent, APP_INK);
    }

    /// The pairs that must read at 4.5:1 in a palette: white (or the accent's own text) on every
    /// accent fill, accent text on every surface and tint, body text on the tints.
    fn assert_readable(t: &Tokens, what: &str) {
        let mut pairs = vec![
            ("on_accent on accent", t.on_accent, t.accent),
            ("on_accent on accent_hover", t.on_accent, t.accent_hover),
            ("on_accent on badge_new", t.on_accent, t.badge_new),
            ("text on accent_soft", t.text, t.accent_soft),
            ("text on selected", t.text, t.selected),
        ];
        for (name, bg) in [
            ("titlebar", t.titlebar),
            ("chrome", t.chrome),
            ("panel", t.panel),
            ("card", t.card),
            ("pasteboard", t.pasteboard),
            ("field", t.field),
            ("accent_soft", t.accent_soft),
            ("selected", t.selected),
        ] {
            pairs.push((name, t.accent_text, bg));
        }
        for (name, fg, bg) in pairs {
            let r = contrast_ratio(fg, bg);
            assert!(r >= 4.5, "{what} {:?}: {name}: {} on {} is {r:.2}:1", t.kind, to_hex(fg), to_hex(bg));
        }
    }

    #[test]
    fn the_nexus_palettes_are_readable() {
        for kind in [ThemeKind::Light, ThemeKind::Dark] {
            assert_readable(&Tokens::for_kind(kind), "app colour");
        }
        // The ink shades really are what white text needs on Rose.
        assert!(contrast_ratio(Color32::WHITE, APP_COLOR) < 4.5);
        assert!(contrast_ratio(Color32::WHITE, APP_INK) >= 4.5);
        assert!(contrast_ratio(Color32::WHITE, APP_INK_DEEP) >= 4.5);
        // Page overlays on the white page and the information blue, error red read as text.
        for c in [SELECTION, INFO, ERROR] {
            assert!(contrast_ratio(c, Color32::WHITE) >= 4.5, "{}", to_hex(c));
        }
        for c in [WARNING, SUCCESS] {
            assert!(contrast_ratio(c, Color32::WHITE) >= 3.0, "{}", to_hex(c));
        }
    }

    /// Any accent colour the user picks keeps every text pair readable, in both themes: the
    /// presets, the extremes and a 6×6×6 grid of the colour cube (216 colours).
    #[test]
    fn user_accents_stay_readable() {
        let mut picks: Vec<Color32> = ACCENT_PRESETS.iter().map(|p| p.2).collect();
        picks.extend([Color32::WHITE, Color32::BLACK, Color32::YELLOW, Color32::from_rgb(0, 0, 0x40), Color32::from_gray(0x80), APP_INK]);
        for r in (0..=255u8).step_by(51) {
            for g in (0..=255u8).step_by(51) {
                for b in (0..=255u8).step_by(51) {
                    picks.push(Color32::from_rgb(r, g, b));
                }
            }
        }
        assert!(picks.len() >= 216 + 6);
        for c in picks {
            for kind in [ThemeKind::Light, ThemeKind::Dark] {
                let t = with_accent(Tokens::for_kind(kind), c);
                assert_readable(&t, &to_hex(c));
                // The surfaces are the theme's own: only the accent family changes.
                let base = Tokens::for_kind(kind);
                assert_eq!((t.chrome, t.panel, t.text, t.border), (base.chrome, base.panel, base.text, base.border));
            }
        }
    }

    /// Information bars keep their neutral tint and icon whatever the accent (and the app colour
    /// Rose) is, unlike the soft accent tint; their text and icon read at 4.5:1.
    #[test]
    fn info_notices_ignore_the_accent() {
        for kind in [ThemeKind::Light, ThemeKind::Dark] {
            let base = Tokens::for_kind(kind);
            assert_ne!(base.info_bg, base.accent_soft, "{kind:?}: the info bar takes the app colour's tint");
            assert!(contrast_ratio(base.text, base.info_bg) >= 4.5, "{kind:?}");
            assert!(contrast_ratio(base.info_icon, base.info_bg) >= 4.5, "{kind:?}");
            for c in ACCENT_PRESETS.iter().map(|p| p.2).chain([APP_COLOR, APP_INK, Color32::BLACK, Color32::WHITE]) {
                let t = with_accent(base, c);
                assert_eq!((t.info_bg, t.info_icon), (base.info_bg, base.info_icon), "{kind:?} {}", to_hex(c));
                assert_ne!(t.info_bg, t.accent_soft, "{kind:?} {}", to_hex(c));
            }
        }
    }

    #[test]
    fn hex_colours_parse_strictly() {
        assert_eq!(parse_hex("#f43f5e"), Some(APP_COLOR));
        assert_eq!(parse_hex(" 6366F1 "), Some(Color32::from_rgb(0x63, 0x66, 0xF1)));
        for junk in ["", "#", "#12345", "#1234567", "zzzzzz", "#ÿÿÿ", "ÿÿÿÿÿÿ", "+12345", "#-12345", "##123456", "#12 345", "red", "#１２３４５６"]
        {
            assert_eq!(parse_hex(junk), None, "{junk:?}");
        }
        assert_eq!(to_hex(Color32::from_rgb(1, 2, 255)), "#0102FF");
        for (_, _, c) in ACCENT_PRESETS {
            assert_eq!(parse_hex(&to_hex(c)), Some(c));
        }
    }

    /// The accent lives in the context: setting it changes the palette widgets read and the
    /// installed visuals, in the theme on screen; clearing it goes back to the app colour.
    #[test]
    fn the_accent_is_per_context_and_follows_the_theme() {
        let ctx = egui::Context::default();
        apply(&ctx, ThemeKind::Light);
        let other = egui::Context::default();
        apply(&other, ThemeKind::Light);
        let indigo = Color32::from_rgb(0x63, 0x66, 0xF1);
        assert!(set_accent(&ctx, Some(indigo)));
        assert!(!set_accent(&ctx, Some(indigo)), "no change");
        let t = Tokens::get(&ctx);
        assert_eq!(t, with_accent(Tokens::for_kind(ThemeKind::Light), indigo));
        assert_eq!(ctx.global_style().visuals.selection.bg_fill, t.accent_soft);
        assert_eq!(Tokens::get(&other), Tokens::for_kind(ThemeKind::Light), "another window keeps its own");
        apply(&ctx, ThemeKind::Dark);
        assert_eq!(Tokens::get(&ctx), with_accent(Tokens::for_kind(ThemeKind::Dark), indigo), "switching theme keeps the accent");
        assert!(set_accent(&ctx, None));
        assert_eq!(Tokens::get(&ctx), Tokens::for_kind(ThemeKind::Dark));
    }
}
