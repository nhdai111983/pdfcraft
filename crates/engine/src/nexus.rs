//! NexusOffice branding: the one place the names, ids and folders of Nexus PDF live, so the
//! files this fork changes in upstream PdfCraft only point here (the fork's
//! `nexus/BRANDING_PATCH.md` lists every upstream file touched).
//!
//! What stays upstream's on purpose (spec §4.4): crate names, `PDFCRAFT_*` variables, URL and
//! `postMessage` schemes, file-format keys (`/PCUndrawn`, `%PdfCraft` content markers), i18n keys,
//! log targets and file names, thread and temp-dir names, MCP `serverInfo.name` (a machine
//! identifier clients configure by) and schema names.
//!
//! Interface text that names upstream's product goes through the interface catalog lookup, which
//! brands it (`pdfcraft_ui_egui::i18n::brand`). Text that ends up in people's files or in other
//! programs (PDF metadata, signature dictionaries, comment authors, CLI and MCP text) is written
//! with the names below at its source, because it never passes that lookup.

/// The product name: window title, menus, About, messages, PDF Producer and Creator.
pub const APP_NAME: &str = "Nexus PDF";
/// The suite the app belongs to.
pub const SUITE_NAME: &str = "NexusOffice";
/// The publisher (installer Manufacturer, package vendor).
pub const PUBLISHER: &str = "SonnyTech";
/// Desktop app id: Wayland app id / X11 `WM_CLASS`, macOS bundle id, the Linux `.desktop` file
/// and hicolor icon name. Never upstream's `ai.storyteller.pdfcraft`.
pub const APP_ID: &str = "vn.sonnytech.nexus.pdf";
/// The installed desktop program (`nexus-pdf`, `nexus-pdf.exe`); the Cargo binary is still
/// `pdfcraft`, the packages install it under this name.
pub const APP_BIN: &str = "nexus-pdf";
/// The installed command-line program (`nexus-pdf-cli`).
pub const CLI_BIN: &str = "nexus-pdf-cli";
/// The per-user folder name on macOS (`~/Library/Application Support/…`, `~/Library/Caches/…`)
/// and Windows (`%APPDATA%\…`, `%LOCALAPPDATA%\…`): Nexus PDF's own, so it never reads, moves or
/// overwrites PdfCraft's settings, logs, crash-recovery files or running instance. No space, so
/// `eframe::storage_dir` and the hand-built paths (recovery, clipboard, digital IDs, the
/// single-instance socket) name the same folder, as upstream's `PdfCraft` does.
pub const SETTINGS_DIR: &str = "NexusPDF";
/// The per-user folder name under the XDG base directories (Linux, BSD):
/// `$XDG_CONFIG_HOME/nexuspdf`, `$XDG_DATA_HOME/nexuspdf`, `$XDG_RUNTIME_DIR/nexuspdf`: the
/// lowercase of [`SETTINGS_DIR`], which is also what `eframe::storage_dir` makes of it there.
pub const SETTINGS_DIR_XDG: &str = "nexuspdf";
/// The data folder a portable copy keeps beside its executable.
pub const PORTABLE_DATA_DIR: &str = "NexusPDFData";
/// The Nexus marker file (besides `portable.txt`) that switches on portable mode.
pub const PORTABLE_MARKER: &str = "NexusPDF.portable";
/// The default author of comments and marks added without one (MCP, `run` scripts): it is written
/// into the document.
pub const DEFAULT_AUTHOR: &str = "Nexus PDF User";
/// Where this build's source lives. (The update check still asks the repository's old address,
/// `nhdai111983/pdfcraft`, which GitHub redirects once the repository is transferred.)
pub const SOURCE_URL: &str = "https://github.com/sonnytech-vn/pdfcraft";

/// The About page's attribution to upstream, which the MIT/Apache licences and ArtCraft's brand
/// licence ask for: text only, no logo, no endorsement implied. English, Vietnamese. Never
/// branded (it is not looked up through the interface catalog).
pub const ATTRIBUTION: (&str, &str) = (
    "NexusOffice is developed from the open-source apps of ArtCraft (MIT/Apache-2.0 licences). Nexus PDF is based on PdfCraft.",
    "NexusOffice được phát triển dựa trên các ứng dụng mã nguồn mở của ArtCraft (giấy phép MIT/Apache-2.0). Nexus PDF dựa trên PdfCraft.",
);
/// Upstream's copyright notice, kept as upstream wrote it (`apps/pdfcraft/build.rs`). Never
/// branded.
pub const UPSTREAM_COPYRIGHT: &str = "Copyright (c) the PdfCraft contributors. MIT OR Apache-2.0.";
/// The About page's one-line suite sentence (English, Vietnamese).
pub const ABOUT_SUITE: (&str, &str) = ("Part of the NexusOffice suite by SonnyTech.", "Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech.");
/// Nexus PDF's own notice.
pub const COPYRIGHT: &str = "Nexus PDF © 2026 SonnyTech. MIT OR Apache-2.0.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_branding_table() {
        assert_eq!(APP_NAME, "Nexus PDF");
        assert_eq!(SUITE_NAME, "NexusOffice");
        assert_eq!(PUBLISHER, "SonnyTech");
        assert_eq!(APP_ID, "vn.sonnytech.nexus.pdf");
        assert_eq!(SOURCE_URL, "https://github.com/sonnytech-vn/pdfcraft");
        assert_eq!(ABOUT_SUITE.0, "Part of the NexusOffice suite by SonnyTech.");
        assert_eq!(ABOUT_SUITE.1, "Thuộc bộ ứng dụng văn phòng NexusOffice của SonnyTech.");
        assert_eq!((APP_BIN, CLI_BIN), ("nexus-pdf", "nexus-pdf-cli"));
        assert_eq!(DEFAULT_AUTHOR, "Nexus PDF User");
        // Never upstream's folders, so the two apps keep their settings apart.
        for dir in [SETTINGS_DIR, SETTINGS_DIR_XDG, PORTABLE_DATA_DIR, PORTABLE_MARKER] {
            assert!(!dir.to_ascii_lowercase().contains("pdfcraft"), "{dir}");
        }
        assert!(!SOURCE_URL.contains("storytold") && !SOURCE_URL.contains("artcraft"));
    }

    /// eframe's `storage_dir` normalises an app name (macOS keeps it but swaps spaces, Linux
    /// lowercases and strips them), while recovery, the clipboard cache, digital IDs and the
    /// single-instance socket build their paths by hand from these constants. A name with no
    /// whitespace whose XDG form is its lowercase survives that normalisation unchanged on every
    /// platform, as upstream's `PdfCraft`/`pdfcraft` does, so there is one folder, not two.
    #[test]
    fn settings_dir_survives_eframe_normalisation() {
        assert!(!SETTINGS_DIR.chars().any(char::is_whitespace), "{SETTINGS_DIR:?}");
        assert!(SETTINGS_DIR.chars().all(|c| c.is_ascii_alphanumeric()), "{SETTINGS_DIR:?}");
        assert_eq!(SETTINGS_DIR_XDG, SETTINGS_DIR.to_lowercase());
    }

    /// About attributes upstream in plain text (no logo, no link to ArtCraft) in both languages.
    #[test]
    fn attribution_is_text_only_and_names_upstream() {
        for (text, lang) in [(ATTRIBUTION.0, "en"), (ATTRIBUTION.1, "vi")] {
            assert!(text.contains("ArtCraft") && text.contains("PdfCraft") && text.contains("MIT/Apache-2.0"), "{lang}: {text}");
            assert!(!text.contains("http"), "{lang}: text only");
        }
        assert!(UPSTREAM_COPYRIGHT.contains("PdfCraft contributors"), "upstream's copyright kept");
    }
}
